use anyhow::{anyhow, bail, Result};
use core::default::Default;
use dialoguer::{theme::ColorfulTheme, Confirm};
use sailfish::TemplateOnce;
use semver::Version;
use std::{fmt, fs, path::PathBuf};

use crate::{fl, println_with_prefix, println_with_prefix_and_fl, Config, REL_DEST_PATH};

const SRC_PATH: &str = "/boot/";
const UCODE: &str = "intel-ucode.img";
const MODULES_PATH: &str = "/usr/lib/modules/";
const REL_ENTRY_PATH: &str = "loader/entries/";

#[derive(TemplateOnce)]
#[template(path = "entry.stpl")]
struct Entry<'a> {
    distro: &'a str,
    kernel: &'a str,
    vmlinuz: &'a str,
    ucode: Option<&'a str>,
    initrd: Option<&'a str>,
    options: &'a str,
}

/// A kernel struct for parsing kernel filenames
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Kernel {
    version: Version,
    localversion: String,
    vmlinuz: String,
    initrd: String,
    distro: String,
    esp_mountpoint: PathBuf,
    bootarg: String,
}

impl Ord for Kernel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for Kernel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Kernel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.version, self.localversion)
    }
}

impl Kernel {
    /// Parse a kernel filename
    pub fn parse(config: &Config, kernel_name: &str) -> Result<Self> {
        // Split the kernel filename into 2 parts in order to determine
        // the version and the localversion of the kernel
        let mut splitted_kernel_name = kernel_name.splitn(2, '-');
        let version = Version::parse(
            splitted_kernel_name
                .next()
                .ok_or_else(|| anyhow!(fl!("invalid_kernel_filename")))?,
        )?;
        let localversion = splitted_kernel_name.next().unwrap_or("unknown").to_owned();

        let vmlinuz = config
            .vmlinuz
            .replace("{VERSION}", &version.to_string())
            .replace("{LOCALVERSION}", &localversion);
        let initrd = config
            .initrd
            .replace("{VERSION}", &version.to_string())
            .replace("{LOCALVERSION}", &localversion);

        Ok(Self {
            version,
            localversion,
            vmlinuz,
            initrd,
            distro: config.distro.to_owned(),
            esp_mountpoint: config.esp_mountpoint.to_owned(),
            bootarg: config.bootarg.to_owned(),
        })
    }

    /// Generate a sorted vector of kernel filenames
    pub fn list_kernels(config: &Config) -> Result<Vec<Self>> {
        // read /usr/lib/modules to get kernel filenames
        let mut kernels = Vec::new();
        for f in fs::read_dir(MODULES_PATH)? {
            let dirname = f?.file_name().into_string().unwrap();
            let dirpath = PathBuf::from(MODULES_PATH).join(&dirname);
            if dirpath.join("modules.dep").exists()
                && dirpath.join("modules.order").exists()
                && dirpath.join("modules.builtin").exists()
            {
                kernels.push(Self::parse(config, &dirname)?);
            } else {
                println_with_prefix_and_fl!("skip_incomplete_kernel", kernel = dirname);
            }
        }
        // Sort the vector, thus the kernel filenames are
        // arranged with versions from newer to older
        kernels.sort_by(|a, b| b.cmp(a));

        Ok(kernels)
    }

    /// Install a specific kernel to the esp using the given kernel filename
    pub fn install(&self) -> Result<()> {
        // if the path does not exist, ask the user for initializing friend
        let dest_path = self.esp_mountpoint.join(REL_DEST_PATH);
        let src_path = PathBuf::from(SRC_PATH);
        if !dest_path.exists() {
            println_with_prefix_and_fl!("info_path_not_exist");
            bail!(
                "{}",
                fl!("err_path_not_exist", path = dest_path.to_string_lossy())
            );
        }
        // generate the path to the source files
        println_with_prefix_and_fl!(
            "install",
            kernel = self.to_string(),
            path = dest_path.to_string_lossy()
        );
        // Copy the source files to the `install_path` using specific
        // filename format, remove the version parts of the files
        fs::copy(src_path.join(&self.vmlinuz), dest_path.join(&self.vmlinuz))?;
        fs::copy(src_path.join(&self.initrd), dest_path.join(&self.initrd)).ok();
        // copy Intel ucode if exists
        let ucode_path = src_path.join(UCODE);
        if ucode_path.exists() {
            println_with_prefix_and_fl!("install_ucode");
            fs::copy(ucode_path, dest_path.join(UCODE))?;
        }

        Ok(())
    }

    /// Create a systemd-boot entry config
    pub fn make_config(&self, force_write: bool) -> Result<()> {
        // if the path does not exist, ask the user for initializing friend
        let entries_path = self.esp_mountpoint.join(REL_ENTRY_PATH);
        if !entries_path.exists() {
            println_with_prefix_and_fl!("info_path_not_exist");
            bail!(
                "{}",
                fl!("err_path_not_exist", path = entries_path.to_string_lossy())
            );
        }
        let entry_path = entries_path.join(self.to_string() + ".conf");
        // do not override existed entry file until forced to do so
        if entry_path.exists() && !force_write {
            let overwrite = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(fl!("ask_overwrite", entry = entry_path.to_string_lossy()))
                .default(false)
                .interact()?;
            if !overwrite {
                println_with_prefix_and_fl!("no_overwrite");
                return Ok(());
            }
            println_with_prefix_and_fl!("overwrite", entry = entry_path.to_string_lossy());
            self.make_config(overwrite)?;
            return Ok(());
        }
        println_with_prefix_and_fl!(
            "create_entry",
            kernel = self.to_string(),
            path = entry_path.to_string_lossy()
        );
        // Generate entry config
        let dest_path = self.esp_mountpoint.join(REL_DEST_PATH);
        fs::write(
            entry_path,
            Entry {
                distro: &self.distro,
                kernel: &self.to_string(),
                vmlinuz: &self.vmlinuz,
                ucode: if dest_path.join(UCODE).exists() {
                    Some(UCODE)
                } else {
                    None
                },
                initrd: if dest_path.join(&self.initrd).exists() {
                    Some(&self.initrd)
                } else {
                    None
                },
                options: &self.bootarg,
            }
            .render_once()?,
        )?;

        Ok(())
    }

    // Try to remove a kernel
    pub fn remove(&self) -> Result<()> {
        let kernel_path = self.esp_mountpoint.join(REL_DEST_PATH);
        println_with_prefix_and_fl!("remove_kernel", kernel = self.to_string());
        fs::remove_file(kernel_path.join(&self.vmlinuz))?;
        fs::remove_file(kernel_path.join(&self.initrd))?;
        println_with_prefix_and_fl!("remove_entry", kernel = self.to_string());
        fs::remove_file(
            self.esp_mountpoint
                .join(format!("loader/entries/{}.conf", self)),
        )?;

        Ok(())
    }

    #[inline]
    pub fn install_and_make_config(&self, force_write: bool) -> Result<()> {
        self.install()?;
        self.make_config(force_write)?;

        Ok(())
    }
}

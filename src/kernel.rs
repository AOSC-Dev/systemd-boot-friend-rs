use anyhow::{anyhow, bail, Result};
use dialoguer::Confirm;
use regex::Regex;

use std::{
    cmp::Ordering,
    fmt, fs,
    io::{prelude::*, BufWriter},
    path::PathBuf,
};
use systemd_boot_conf::SystemdBootConf;

use crate::{
    fl, parser::Version, print_block_with_fl, println_with_prefix, println_with_prefix_and_fl,
    Config, REL_DEST_PATH,
};

const SRC_PATH: &str = "/boot/";
const UCODE: &str = "intel-ucode.img";
const MODULES_PATH: &str = "/usr/lib/modules/";
const REL_ENTRY_PATH: &str = "loader/entries/";

/// A kernel struct for parsing kernel filenames
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Kernel {
    version: Version,
    vmlinuz: String,
    initrd: String,
    distro: String,
    esp_mountpoint: PathBuf,
    entry: String,
    bootarg: String,
}

impl Ord for Kernel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for Kernel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Kernel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)
    }
}

impl Kernel {
    /// Parse a kernel filename
    pub fn parse(config: &Config, kernel_name: &str) -> Result<Self> {
        let version = Version::parse(kernel_name)?;
        let vmlinuz = config.vmlinuz.replace("{VERSION}", kernel_name);
        let initrd = config.initrd.replace("{VERSION}", kernel_name);
        let entry = kernel_name.to_owned() + ".conf";

        Ok(Self {
            version,
            vmlinuz,
            initrd,
            distro: config.distro.to_owned(),
            esp_mountpoint: config.esp_mountpoint.to_owned(),
            entry,
            bootarg: config.bootarg.to_owned(),
        })
    }

    /// Generate a sorted vector of kernel filenames
    pub fn list(config: &Config) -> Result<Vec<Self>> {
        // read /usr/lib/modules to get kernel filenames
        let mut kernels = Vec::new();

        for f in fs::read_dir(MODULES_PATH)? {
            let dirname = f?.file_name().into_string().unwrap();
            let dirpath = PathBuf::from(MODULES_PATH).join(&dirname);

            if dirpath.join("modules.dep").exists()
                && dirpath.join("modules.order").exists()
                && dirpath.join("modules.builtin").exists()
            {
                match Self::parse(config, &dirname) {
                    Ok(k) => kernels.push(k),
                    Err(_) => {
                        println_with_prefix_and_fl!("skip_unidentified_kernel", kernel = dirname);
                    }
                }
            } else {
                println_with_prefix_and_fl!("skip_incomplete_kernel", kernel = dirname);
            }
        }

        // Sort the vector, thus the kernels are
        // arranged with versions from newer to older
        kernels.sort_by(|a, b| b.cmp(a));

        Ok(kernels)
    }

    /// Generate installed kernel list
    pub fn list_installed(config: &Config) -> Result<Vec<Self>> {
        let mut installed_kernels = Vec::new();

        // Construct regex for the template
        let re = Regex::new(&config.vmlinuz.replace("{VERSION}", r"(?P<version>.+)"))?;

        // Regex match group
        if let Ok(d) = fs::read_dir(config.esp_mountpoint.join(REL_DEST_PATH)) {
            for x in d {
                let filename = &x?
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow!(fl!("invalid_kernel_filename")))?;

                if let Some(c) = re.captures(filename) {
                    let version = c
                        .name("version")
                        .ok_or_else(|| anyhow!(fl!("invalid_kernel_filename")))?
                        .as_str();

                    installed_kernels.push(Self::parse(config, version)?);
                }
            }
        }

        // Sort the vector, thus the kernels are
        // arranged with versions from newer to older
        installed_kernels.sort_by(|a, b| b.cmp(a));

        Ok(installed_kernels)
    }

    /// Install a specific kernel to the esp using the given kernel filename
    pub fn install(&self) -> Result<()> {
        // if the path does not exist, ask the user for initializing friend
        let dest_path = self.esp_mountpoint.join(REL_DEST_PATH);
        let src_path = PathBuf::from(SRC_PATH);

        if !dest_path.exists() {
            print_block_with_fl!("info_path_not_exist");
            bail!(fl!(
                "err_path_not_exist",
                path = dest_path.to_string_lossy()
            ));
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
        let ucode_dest_path = dest_path.join(UCODE);

        if ucode_path.exists() {
            println_with_prefix_and_fl!("install_ucode");
            fs::copy(ucode_path, ucode_dest_path)?;
        } else {
            fs::remove_file(ucode_dest_path).ok();
        }

        Ok(())
    }

    /// Create a systemd-boot entry config
    pub fn make_config(&self, force_write: bool) -> Result<()> {
        // if the path does not exist, ask the user for initializing friend
        let entries_path = self.esp_mountpoint.join(REL_ENTRY_PATH);

        if !entries_path.exists() {
            print_block_with_fl!("info_path_not_exist");
            bail!(fl!(
                "err_path_not_exist",
                path = entries_path.to_string_lossy()
            ));
        }

        // do not override existed entry file until forced to do so
        let entry_path = entries_path.join(&self.entry);

        if entry_path.exists() && !force_write {
            let overwrite = Confirm::new()
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

        // Generate entry config
        println_with_prefix_and_fl!("create_entry", kernel = self.to_string());

        let dest_path = self.esp_mountpoint.join(REL_DEST_PATH);
        let rel_dest_path = PathBuf::from(REL_DEST_PATH);
        let file = fs::File::create(entry_path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "title {} ({})", self.distro, self)?;
        writeln!(
            writer,
            "linux /{}",
            rel_dest_path.join(&self.vmlinuz).display()
        )?;
        dest_path
            .join(UCODE)
            .exists()
            .then(|| writeln!(writer, "initrd /{}{}", REL_DEST_PATH, UCODE))
            .transpose()?;
        dest_path
            .join(&self.initrd)
            .exists()
            .then(|| writeln!(writer, "initrd /{}{}", REL_DEST_PATH, self.initrd))
            .transpose()?;
        writeln!(writer, "options {}", self.bootarg)?;

        Ok(())
    }

    // Try to remove a kernel
    pub fn remove(&self) -> Result<()> {
        let kernel_path = self.esp_mountpoint.join(REL_DEST_PATH);

        println_with_prefix_and_fl!("remove_kernel", kernel = self.to_string());
        fs::remove_file(kernel_path.join(&self.vmlinuz))?;
        fs::remove_file(kernel_path.join(&self.initrd)).ok();

        println_with_prefix_and_fl!("remove_entry", kernel = self.to_string());
        fs::remove_file(
            self.esp_mountpoint
                .join(format!("loader/entries/{}", self.entry)),
        )?;

        self.remove_default()?;

        Ok(())
    }

    // Set default entry
    pub fn set_default(&self) -> Result<()> {
        println_with_prefix_and_fl!("set_default", kernel = self.to_string());

        let mut conf = SystemdBootConf::new(&self.esp_mountpoint)?;

        conf.loader_conf.default = Some(Box::from(self.entry.as_str()));
        conf.overwrite_loader_conf()?;

        Ok(())
    }

    // Remove default entry
    pub fn remove_default(&self) -> Result<()> {
        let mut conf = SystemdBootConf::new(&self.esp_mountpoint)?;

        if conf.loader_conf.default == Some(Box::from(self.entry.as_str())) {
            println_with_prefix_and_fl!("remove_default", kernel = self.to_string());
            conf.loader_conf.default = None;
            conf.overwrite_loader_conf()?;
        }

        Ok(())
    }

    #[inline]
    pub fn ask_set_default(&self) -> Result<()> {
        Confirm::new()
            .with_prompt(fl!("ask_set_default", kernel = self.to_string()))
            .default(false)
            .interact()?
            .then(|| self.set_default())
            .transpose()?;

        Ok(())
    }

    #[inline]
    pub fn install_and_make_config(&self, force_write: bool) -> Result<()> {
        self.install()?;
        self.make_config(force_write)?;

        Ok(())
    }
}

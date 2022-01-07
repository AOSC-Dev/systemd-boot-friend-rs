use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use lazy_static::lazy_static;
use regex::Regex;
use std::{fmt, fs, path::PathBuf};

use crate::{fl, println_with_prefix, println_with_prefix_and_fl, Config, REL_DEST_PATH};

const SRC_PATH: &str = "/boot/";
const UCODE: &str = "intel-ucode.img";
const MODULES_PATH: &str = "/usr/lib/modules/";
const REL_ENTRY_PATH: &str = "loader/entries/";
lazy_static! {
    static ref KERNEL_REGEX: Regex = Regex::new(
            r"(?P<major>[0-9]+)\.(?P<minor>[0-9]+)\.(?P<patch>[0-9]+)-((?P<rc>rc[0-9]+)?-|)((?P<rel>[0-9]+)?-|)(?P<localversion>.+)"
        )
        .unwrap();
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    rc: Option<String>,
    rel: Option<u64>,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}{}{}",
            self.major,
            self.minor,
            self.patch,
            self.rc
                .as_ref()
                .map_or_else(|| "".to_owned(), |s| format!("-{}", s)),
            self.rel
                .as_ref()
                .map_or_else(|| "".to_owned(), |s| format!("-{}", s)),
        )
    }
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

pub fn parse_kernel_name(kernel_name: &str) -> Result<(Version, String)> {
    if let Some(cap) = KERNEL_REGEX.captures(kernel_name) {
        let version = Version {
            // These are safe because I'm good
            major: cap.name("major").unwrap().as_str().parse()?,
            minor: cap.name("minor").unwrap().as_str().parse()?,
            patch: cap.name("patch").unwrap().as_str().parse()?,
            rc: cap.name("rc").map(|m| m.as_str().to_owned()),
            rel: cap.name("rel").map(|m| m.as_str().parse().unwrap()),
        };
        let localversion = cap.name("localversion").unwrap().as_str().to_owned();
        Ok((version, localversion))
    } else {
        bail!(fl!("invalid_kernel_filename"))
    }
}

impl Kernel {
    /// Parse a kernel filename
    pub fn parse(config: &Config, kernel_name: &str) -> Result<Self> {
        let vmlinuz;
        let initrd;
        let (version, localversion) = parse_kernel_name(kernel_name)?;
        vmlinuz = config.vmlinuz.replace("{VERSION}", kernel_name);
        initrd = config.initrd.replace("{VERSION}", kernel_name);

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
                if let Ok(k) = Self::parse(config, &dirname) {
                    kernels.push(k);
                } else {
                    println_with_prefix_and_fl!("skip_unidentified_kernel", kernel = dirname);
                }
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
        if ucode_path.exists() {
            println_with_prefix_and_fl!("install_ucode");
            fs::copy(ucode_path, dest_path.join(UCODE))?;
        } else {
            fs::remove_file(dest_path.join(UCODE)).ok();
        }

        Ok(())
    }

    /// Create a systemd-boot entry config
    pub fn make_config(&self, force_write: bool) -> Result<()> {
        // if the path does not exist, ask the user for initializing friend
        let entries_path = self.esp_mountpoint.join(REL_ENTRY_PATH);
        if !entries_path.exists() {
            println_with_prefix_and_fl!("info_path_not_exist");
            bail!(fl!(
                "err_path_not_exist",
                path = entries_path.to_string_lossy()
            ));
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
        let rel_dest_path = PathBuf::from(REL_DEST_PATH);
        let entry = format!(
            "title {} ({})\nlinux /{}\n{}{}options {}",
            &self.distro,
            &self.to_string(),
            rel_dest_path.join(&self.vmlinuz).display(),
            if dest_path.join(UCODE).exists() {
                format!("initrd /{}{}\n", REL_DEST_PATH, UCODE)
            } else { String::new() },
            if dest_path.join(&self.initrd).exists() {
                format!("initrd /{}{}\n", REL_DEST_PATH, &self.initrd)
            } else { String::new() },
            &self.bootarg
        );
        fs::write(entry_path, entry)?;

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

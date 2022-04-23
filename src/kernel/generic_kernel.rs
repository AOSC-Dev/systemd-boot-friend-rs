use anyhow::{anyhow, bail, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use libsdbootconf::{
    entry::{EntryBuilder, Token},
    SystemdBootConf,
};
use regex::Regex;
use std::{cell::RefCell, cmp::Ordering, fmt, fs, path::PathBuf, rc::Rc};

use super::{file_copy, Kernel, REL_ENTRY_PATH};
use crate::{
    fl, print_block_with_fl, println_with_prefix, println_with_prefix_and_fl,
    version::{generic_version::GenericVersion, Version},
    Config, REL_DEST_PATH, SRC_PATH,
};

const MODULES_PATH: &str = "/usr/lib/modules/";
const UCODE: &str = "intel-ucode.img";

/// A kernel struct for parsing kernel filenames
#[derive(Debug, Clone)]
pub struct GenericKernel {
    version: GenericVersion,
    vmlinux: String,
    initrd: String,
    distro: Rc<String>,
    esp_mountpoint: Rc<PathBuf>,
    entry: String,
    bootarg: Rc<String>,
    sbconf: Rc<RefCell<SystemdBootConf>>,
}

impl PartialEq for GenericKernel {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.vmlinux == other.vmlinux
            && self.initrd == other.initrd
            && self.distro == other.distro
            && self.esp_mountpoint == other.esp_mountpoint
            && self.entry == other.entry
            && self.bootarg == other.bootarg
    }
}

impl Eq for GenericKernel {}

impl Ord for GenericKernel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for GenericKernel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for GenericKernel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)
    }
}

impl Kernel for GenericKernel {
    /// Install a specific kernel to the esp using the given kernel filename
    fn install(&self) -> Result<()> {
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
        println_with_prefix_and_fl!("install", kernel = self.to_string());

        // Copy the source files to the `install_path` using specific
        // filename format, remove the version parts of the files
        file_copy(src_path.join(&self.vmlinux), dest_path.join(&self.vmlinux))?;

        let initrd_path = src_path.join(&self.initrd);

        if initrd_path.exists() {
            file_copy(src_path.join(&self.initrd), dest_path.join(&self.initrd))?;
        }

        // copy Intel ucode if exists
        let ucode_path = src_path.join(UCODE);
        let ucode_dest_path = dest_path.join(UCODE);

        if ucode_path.exists() {
            println_with_prefix_and_fl!("install_ucode");
            file_copy(ucode_path, ucode_dest_path)?;
        } else {
            fs::remove_file(ucode_dest_path).ok();
        }

        Ok(())
    }

    // Try to remove a kernel
    fn remove(&self) -> Result<()> {
        let kernel_path = self.esp_mountpoint.join(REL_DEST_PATH);

        println_with_prefix_and_fl!("remove_kernel", kernel = self.to_string());
        let vmlinux = kernel_path.join(&self.vmlinux);
        let initrd = kernel_path.join(&self.initrd);

        fs::remove_file(&vmlinux)
            .map_err(|x| eprintln!("WARNING: {}: {}", &vmlinux.display(), x))
            .ok();
        fs::remove_file(&initrd)
            .map_err(|x| eprintln!("WARNING: {}: {}", &initrd.display(), x))
            .ok();

        println_with_prefix_and_fl!("remove_entry", kernel = self.to_string());
        let entry = self
            .esp_mountpoint
            .join(format!("loader/entries/{}.conf", self.entry));

        fs::remove_file(&entry)
            .map_err(|x| eprintln!("WARNING: {}: {}", &entry.display(), x))
            .ok();

        self.remove_default()?;

        Ok(())
    }

    /// Create a systemd-boot entry config
    fn make_config(&self, force_write: bool) -> Result<()> {
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

        // Generate entry config
        println_with_prefix_and_fl!("create_entry", kernel = self.to_string());

        let dest_path = self.esp_mountpoint.join(REL_DEST_PATH);
        let rel_dest_path = PathBuf::from(REL_DEST_PATH);

        let mut entry = EntryBuilder::new(&self.entry)
            .title(format!("{} ({})", self.distro, self))
            .linux(rel_dest_path.join(&self.vmlinux))
            .build();

        dest_path
            .join(UCODE)
            .exists()
            .then(|| entry.tokens.push(Token::Initrd(rel_dest_path.join(UCODE))));
        dest_path.join(&self.initrd).exists().then(|| {
            entry
                .tokens
                .push(Token::Initrd(rel_dest_path.join(&self.initrd)))
        });
        entry
            .tokens
            .push(Token::Options((*self.bootarg).to_owned()));
        self.sbconf.borrow_mut().entries.push(entry);
        self.sbconf.borrow().write_entries()?;

        Ok(())
    }

    // Set default entry
    fn set_default(&self) -> Result<()> {
        println_with_prefix_and_fl!("set_default", kernel = self.to_string());
        self.sbconf.borrow_mut().config.default = Some(self.entry.to_owned());
        self.sbconf.borrow().write_config()?;

        Ok(())
    }

    // Remove default entry
    fn remove_default(&self) -> Result<()> {
        if self.sbconf.borrow().config.default.as_ref() == Some(&self.entry) {
            println_with_prefix_and_fl!("remove_default", kernel = self.to_string());
            self.sbconf.borrow_mut().config.default = None;
            self.sbconf.borrow().write_config()?;
        }

        Ok(())
    }

    #[inline]
    fn ask_set_default(&self) -> Result<()> {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(fl!("ask_set_default", kernel = self.to_string()))
            .default(false)
            .interact()?
            .then(|| self.set_default())
            .transpose()?;

        Ok(())
    }

    /// Check if the kernel is the default kernel
    #[inline]
    fn is_default(&self) -> bool {
        if let Some(default) = &self.sbconf.borrow().config.default {
            default == &self.to_string()
        } else {
            false
        }
    }

    #[inline]
    fn install_and_make_config(&self, force_write: bool) -> Result<()> {
        self.install()?;
        self.make_config(force_write)?;

        Ok(())
    }
}

impl GenericKernel {
    /// Parse a kernel filename
    pub fn parse(
        config: &Config,
        kernel_name: &str,
        sbconf: Rc<RefCell<SystemdBootConf>>,
    ) -> Result<Self> {
        let version = GenericVersion::parse(kernel_name)?;
        let vmlinux = config.vmlinux.replace("{VERSION}", kernel_name);
        let initrd = config.initrd.replace("{VERSION}", kernel_name);
        let entry = kernel_name.to_owned();
        sbconf.borrow_mut().load_current()?;

        Ok(Self {
            version,
            vmlinux,
            initrd,
            distro: config.distro.clone(),
            esp_mountpoint: config.esp_mountpoint.clone(),
            entry,
            bootarg: config.bootarg.clone(),
            sbconf,
        })
    }

    /// Generate a sorted vector of kernel filenames
    pub fn list(config: &Config, sbconf: Rc<RefCell<SystemdBootConf>>) -> Result<Vec<Rc<Self>>> {
        // read /usr/lib/modules to get kernel filenames
        let mut kernels = Vec::new();

        for f in fs::read_dir(MODULES_PATH)? {
            let dirname = f?
                .file_name()
                .into_string()
                .map_err(|s| anyhow!("fail to parse directory name: {:?}", s))?;
            let dirpath = PathBuf::from(MODULES_PATH).join(&dirname);

            if dirpath.join("modules.dep").exists()
                && dirpath.join("modules.order").exists()
                && dirpath.join("modules.builtin").exists()
            {
                match Self::parse(config, &dirname, sbconf.clone()) {
                    Ok(k) => kernels.push(Rc::new(k)),
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
    pub fn list_installed(
        config: &Config,
        sbconf: Rc<RefCell<SystemdBootConf>>,
    ) -> Result<Vec<Rc<Self>>> {
        let mut installed_kernels = Vec::new();

        // Construct regex for the template
        let re = Regex::new(&config.vmlinux.replace("{VERSION}", r"(?P<version>.+)"))?;

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

                    installed_kernels.push(Rc::new(Self::parse(config, version, sbconf.clone())?));
                }
            }
        }

        // Sort the vector, thus the kernels are
        // arranged with versions from newer to older
        installed_kernels.sort_by(|a, b| b.cmp(a));

        Ok(installed_kernels)
    }
}

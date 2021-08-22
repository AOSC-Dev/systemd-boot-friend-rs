use crate::{println_with_prefix, CONF_PATH, REL_DEST_PATH};
use anyhow::{anyhow, Result};
use core::{default::Default, str::FromStr};
use dialoguer::{theme::ColorfulTheme, Confirm};
use semver::Version;
use std::{fmt, fs, path::Path};

const SRC_PATH: &str = "/boot/";
const UCODE: &str = "intel-ucode.img";
const MODULES_PATH: &str = "/usr/lib/modules/";
const REL_ENTRY_PATH: &str = "loader/entries/";

/// A kernel struct for parsing kernel filenames
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Kernel {
    pub version: Version,
    pub localversion: String,
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

impl FromStr for Kernel {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> Result<Self> {
        // Split the kernel filename into 3 parts in order to determine
        // the version and the localversion of the kernel
        let mut splitted_kernel_name = text.splitn(2, '-');
        let version = Version::parse(
            splitted_kernel_name
                .next()
                .ok_or_else(|| anyhow!("invalid kernel filename"))?,
        )?;
        let localversion = splitted_kernel_name.next().unwrap_or("unknown").to_owned();
        Ok(Self {
            version,
            localversion,
        })
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self {
            version: Version::new(0, 0, 0),
            localversion: "unknown".to_owned(),
        }
    }
}

impl Kernel {
    /// Parse a kernel filename
    pub fn parse(kernel_name: &str) -> Result<Self> {
        Self::from_str(kernel_name)
    }

    /// Parse vmlinuz and initrd filenames
    fn parse_filenames(&self, vmlinuz: &str, initrd: &str) -> (String, String) {
        (
            vmlinuz
                .replace("{VERSION}", &self.version.to_string())
                .replace("{LOCALVERSION}", &self.localversion),
            initrd
                .replace("{VERSION}", &self.version.to_string())
                .replace("{LOCALVERSION}", &self.localversion),
        )
    }

    /// Generate a sorted vector of kernel filenames
    pub fn list_kernels() -> Result<Vec<Self>> {
        // read /usr/lib/modules to get kernel filenames
        let mut kernels = fs::read_dir(MODULES_PATH)?
            .map(|k| {
                Self::parse(
                    &k?.file_name()
                        .into_string()
                        .unwrap_or_else(|_| String::new()),
                )
            })
            .collect::<Result<Vec<Self>>>()?;
        // Sort the vector, thus the kernel filenames are
        // arranged with versions from newer to older
        kernels.sort_by(|a, b| b.cmp(a));
        Ok(kernels)
    }

    /// Install a specific kernel to the esp using the given kernel filename
    pub fn install(&self, vmlinuz: &str, initrd: &str, esp_path: &Path) -> Result<()> {
        // if the path does not exist, ask the user for initializing friend
        let dest_path = esp_path.join(REL_DEST_PATH);
        let src_path = Path::new(SRC_PATH);
        if !dest_path.exists() {
            println_with_prefix!("{} does not exist. Doing nothing.", dest_path.display());
            println_with_prefix!(
                "If you wish to use systemd-boot, execute `systemd-boot-friend init` first."
            );
            println_with_prefix!(
                "Or, if your ESP mountpoint is not at \"{}\", please edit {}.",
                esp_path.display(),
                CONF_PATH
            );
            return Err(anyhow!("{} not found", dest_path.display()));
        }
        // generate the path to the source files
        println_with_prefix!("Installing {} to {} ...", self, dest_path.display());
        let (vmlinuz, initrd) = self.parse_filenames(vmlinuz, initrd);
        // Copy the source files to the `install_path` using specific
        // filename format, remove the version parts of the files
        fs::copy(src_path.join(&vmlinuz), dest_path.join(&vmlinuz))?;
        fs::copy(src_path.join(&initrd), dest_path.join(&initrd))?;
        // copy Intel ucode if exists
        let ucode_path = src_path.join(UCODE);
        if ucode_path.exists() {
            println_with_prefix!("intel-ucode detected. Installing ...");
            fs::copy(ucode_path, dest_path.join(UCODE))?;
        }

        Ok(())
    }

    /// Create a systemd-boot entry config
    pub fn make_config(
        &self,
        vmlinuz: &str,
        initrd: &str,
        distro: &str,
        esp_path: &Path,
        bootarg: &str,
        force_write: bool,
    ) -> Result<()> {
        // if the path does not exist, ask the user for initializing friend
        let entries_path = esp_path.join(REL_ENTRY_PATH);
        if !entries_path.exists() {
            println_with_prefix!("{} does not exist. Doing nothing.", entries_path.display());
            println_with_prefix!(
                "If you wish to use systemd-boot, execute `systemd-boot-friend init` first."
            );
            println_with_prefix!(
                "Or, if your ESP mountpoint is not at \"{}\", please edit {}.",
                esp_path.display(),
                CONF_PATH
            );
            return Err(anyhow!("{} not found", entries_path.display()));
        }
        let entry_path = entries_path.join(self.to_string());
        // do not override existed entry file until forced to do so
        if entry_path.exists() && !force_write {
            let overwrite = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "{} already exists. Overwrite?",
                    entry_path.display()
                ))
                .default(false)
                .interact()?;
            if !overwrite {
                println_with_prefix!("Doing nothing on this file.");
                return Ok(());
            }
            println_with_prefix!("Overwriting {} ...", entry_path.display());
            self.make_config(vmlinuz, initrd, distro, esp_path, bootarg, overwrite)?;
            return Ok(());
        }
        println_with_prefix!(
            "Creating boot entry for {} at {} ...",
            self,
            entry_path.display()
        );
        let (vmlinuz_file, initrd_file) = self.parse_filenames(vmlinuz, initrd);
        // Generate entry config
        let title = format!("title {} ({})\n", distro, self);
        let vmlinuz = format!("linux /{}{}\n", REL_DEST_PATH, vmlinuz_file);
        // automatically detect Intel ucode and write the config
        let ucode = if esp_path.join(REL_DEST_PATH).join(UCODE).exists() {
            format!("initrd /{}{}\n", REL_DEST_PATH, UCODE)
        } else {
            String::new()
        };
        let initrd = format!("initrd /{}{}\n", REL_DEST_PATH, initrd_file);
        let options = format!("options {}", bootarg);
        let content = title + &vmlinuz + &ucode + &initrd + &options;
        fs::write(entry_path, content)?;

        Ok(())
    }

    // Try to remove a kernel
    pub fn remove(&self, vmlinuz: &str, initrd: &str, esp_path: &Path) -> Result<()> {
        let kernel_path = esp_path.join(REL_DEST_PATH);
        let (vmlinuz, initrd) = self.parse_filenames(vmlinuz, initrd);
        println_with_prefix!("Removing {} kernel ...", self);
        fs::remove_file(kernel_path.join(vmlinuz))?;
        fs::remove_file(kernel_path.join(initrd))?;
        println_with_prefix!("Removing {} boot entry ...", self);
        fs::remove_file(esp_path.join(format!("loader/entries/{}.conf", self)))?;

        Ok(())
    }

    #[inline]
    pub fn install_and_make_config(
        &self,
        vmlinuz: &str,
        initrd: &str,
        distro: &str,
        esp_path: &Path,
        bootarg: &str,
        force_write: bool,
    ) -> Result<()> {
        self.install(vmlinuz, initrd, esp_path)?;
        self.make_config(vmlinuz, initrd, distro, esp_path, bootarg, force_write)?;

        Ok(())
    }
}

#[test]
fn test_kernel_struct() {
    assert_eq!(Kernel::parse("0.0.0-unknown").unwrap(), Kernel::default())
}

#[test]
fn test_kernel_display() {
    assert_eq!(
        format!("{}", Kernel::parse("0.0.0-unknown").unwrap()),
        "0.0.0-unknown"
    )
}

use crate::{println_with_prefix, CONF_PATH, REL_DEST_PATH};
use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use semver::Version;
use std::{fmt, fs, path::Path};

const SRC_PATH: &str = "/boot/";
const UCODE: &str = "intel-ucode.img";
const MODULES_PATH: &str = "/usr/lib/modules/";

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
        // Split the kernel filename into 3 parts in order to determine
        // the version, name and the flavor of the kernel
        let mut splitted_kernel_name = kernel_name.splitn(2, '-');
        let version = Version::parse(
            splitted_kernel_name
                .next()
                .ok_or_else(|| anyhow!("invalid kernel filename"))?,
        )?;
        let localversion = splitted_kernel_name
            .next()
            .unwrap_or_else(|| "unknown")
            .to_owned();
        Ok(Self {
            version,
            localversion,
        })
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
    pub fn install(&self, esp_path: &Path) -> Result<()> {
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
        let vmlinuz = format!("vmlinuz-{}", self);
        let initramfs = format!("initramfs-{}.img", self);
        // Copy the source files to the `install_path` using specific
        // filename format, remove the version parts of the files
        fs::copy(src_path.join(&vmlinuz), dest_path.join(&vmlinuz))?;
        fs::copy(src_path.join(&initramfs), dest_path.join(&initramfs))?;
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
        distro: &str,
        esp_path: &Path,
        bootarg: &str,
        force_write: bool,
    ) -> Result<()> {
        let entry_path = esp_path.join(format!("loader/entries/{}.conf", self));
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
            self.make_config(distro, esp_path, bootarg, overwrite)?;
            return Ok(());
        }
        println_with_prefix!(
            "Creating boot entry for {} at {} ...",
            self,
            entry_path.display()
        );
        // Generate entry config
        let title = format!("title {} ({})\n", distro, self);
        let vmlinuz = format!("linux /{}vmlinuz-{}\n", REL_DEST_PATH, self);
        // automatically detect Intel ucode and write the config
        let ucode = if esp_path.join(REL_DEST_PATH).join(UCODE).exists() {
            format!("initrd /{}{}\n", REL_DEST_PATH, UCODE)
        } else {
            String::new()
        };
        let initramfs = format!("initrd /{}initramfs-{}.img\n", REL_DEST_PATH, self);
        let options = format!("options {}", bootarg);
        let content = title + &vmlinuz + &ucode + &initramfs + &options;
        fs::write(entry_path, content)?;

        Ok(())
    }

    pub fn remove(&self, esp_path: &Path) -> Result<()> {
        let kernel_path = esp_path.join(REL_DEST_PATH);
        println_with_prefix!("Removing {} kernel ...", self);
        fs::remove_file(kernel_path.join(format!("vmlinuz-{}", self)))?;
        fs::remove_file(kernel_path.join(format!("initramfs-{}.img", self)))?;
        println_with_prefix!("Removing {} boot entry ...", self);
        fs::remove_file(esp_path.join(format!("loader/entries/{}.conf", self)))?;

        Ok(())
    }

    #[inline]
    pub fn install_and_make_config(
        &self,
        distro: &str,
        esp_path: &Path,
        bootarg: &str,
        force_write: bool,
    ) -> Result<()> {
        self.install(esp_path)?;
        self.make_config(distro, esp_path, bootarg, force_write)?;

        Ok(())
    }
}

#[test]
fn test_kernel_struct() {
    assert_eq!(Kernel::parse("0.0.0-unknown").unwrap(), Kernel::default())
}

#[test]
fn test_kernel_display() {
    assert_eq!(format!("{}", Kernel::parse("0.0.0-unknown").unwrap()), "0.0.0-unknown")
}

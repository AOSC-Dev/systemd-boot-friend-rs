use crate::*;
use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use semver::Version;
use std::{fmt, fs, path::Path};

const CONF_PATH: &str = "/etc/systemd-boot-friend.conf";
const REL_INST_PATH: &str = "EFI/aosc/";
const SRC_PATH: &str = "/boot/";
const UCODE_PATH: &str = "/boot/intel-ucode.img";
const MODULES_PATH: &str = "/usr/lib/modules/";

/// A kernel struct for parsing kernel filenames
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Kernel {
    pub version: Version,
    pub distro: String,
    pub flavor: String,
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
        write!(f, "{}", self.get_name())
    }
}

impl Kernel {
    /// Parse a kernel filename
    pub fn parse(kernel_name: &str) -> Result<Self> {
        // Split the kernel filename into 3 parts in order to determine
        // the version, name and the flavor of the kernel
        let mut splitted_kernel_name = kernel_name.splitn(3, '-');
        let kernel_version;
        let distro_name;
        let kernel_flavor;
        yield_into!(
            (kernel_version, distro_name, kernel_flavor) = splitted_kernel_name,
            "Invalid kernel filename",
            kernel_name
        );
        Ok(Self {
            version: Version::parse(kernel_version)?,
            distro: distro_name.to_string(),
            flavor: kernel_flavor.to_string(),
        })
    }

    /// Generate a sorted vector of kernel filenames
    pub fn list_kernels() -> Result<Vec<Self>> {
        // read /usr/lib/modules to get kernel filenames
        let mut kernels = fs::read_dir(MODULES_PATH)?
            .map(|k| {
                Kernel::parse(
                    &k?.file_name()
                        .into_string()
                        .unwrap_or_else(|_| String::new()),
                )
            })
            .collect::<Result<Vec<Self>>>()?;

        if kernels.is_empty() {
            return Err(anyhow!("No kernel found"));
        }

        // Sort the vector, thus the kernel filenames are
        // arranged with versions from newer to older
        kernels.sort_by(|a, b| b.cmp(a));
        Ok(kernels)
    }

    /// Get the full name of the kernel
    fn get_name(&self) -> String {
        format!("{}-{}-{}", self.version, self.distro, self.flavor)
    }

    /// Install a specific kernel to the esp using the given kernel filename
    pub fn install(&self, esp_path: &Path) -> Result<()> {
        // if the path does not exist, ask the user for initializing friend
        let install_path = esp_path.join(REL_INST_PATH);
        if !install_path.exists() {
            println_with_prefix!("{} does not exist. Doing nothing.", install_path.display());
            println_with_prefix!(
                "If you wish to use systemd-boot, execute `systemd-boot-friend init` first."
            );
            println_with_prefix!(
                "Or, if your ESP mountpoint is not at \"{}\", please edit {}.",
                esp_path.display(),
                CONF_PATH
            );

            return Err(anyhow!("{} not found", install_path.display()));
        }
        // generate the path to the source files
        println_with_prefix!(
            "Installing {} to {} ...",
            self.get_name(),
            install_path.display()
        );
        let vmlinuz_path = format!(
            "{}vmlinuz-{}-{}-{}",
            SRC_PATH, self.version, self.distro, self.flavor
        );
        let initramfs_path = format!(
            "{}initramfs-{}-{}-{}.img",
            SRC_PATH, self.version, self.distro, self.flavor
        );
        let src_vmlinuz = Path::new(&vmlinuz_path);
        let src_initramfs = Path::new(&initramfs_path);
        let src_ucode = Path::new(UCODE_PATH);
        // Copy the source files to the `install_path` using specific
        // filename format, remove the version parts of the files
        unwrap_or_show_error!(
            {
                fs::copy(
                    &src_vmlinuz,
                    install_path.join(format!("vmlinuz-{}-{}", self.distro, self.flavor)),
                )
            },
            "Unable to copy kernel file"
        );
        unwrap_or_show_error!(
            {
                fs::copy(
                    &src_initramfs,
                    install_path.join(format!("initramfs-{}-{}.img", self.distro, self.flavor)),
                )
            },
            "Unable to copy initramfs file"
        );

        // copy Intel ucode if exists
        if src_ucode.exists() {
            println_with_prefix!("intel-ucode detected. Installing ...");
            fs::copy(&src_ucode, install_path.join("intel-ucode.img"))?;
        }

        Ok(())
    }

    /// Create a systemd-boot entry config
    pub fn make_config(&self, esp_path: &Path, bootarg: &str, force_write: bool) -> Result<()> {
        let entry_path = esp_path.join(format!(
            "loader/entries/{}-{}.conf",
            self.distro, self.flavor
        ));
        // do not override existed entry file until forced to do so
        if entry_path.exists() {
            if !force_write {
                let force_write = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!(
                        "{} already exists. Overwrite?",
                        entry_path.display()
                    ))
                    .default(false)
                    .interact()?;
                if !force_write {
                    println_with_prefix!("Doing nothing on this file.");
                    return Ok(());
                }
                self.make_config(esp_path, bootarg, force_write)?;
                return Ok(());
            }
            println_with_prefix!("Overwriting {} ...", entry_path.display());
        }
        println_with_prefix!(
            "Creating boot entry for {} at {} ...",
            self,
            entry_path.display()
        );
        // Generate entry config
        let title = format!("title AOSC OS ({})\n", self.flavor);
        let vmlinuz = format!(
            "linux /{}vmlinuz-{}-{}\n",
            REL_INST_PATH, self.distro, self.flavor
        );
        // automatically detect Intel ucode and write the config
        let mut ucode = String::new();
        if esp_path
            .join(REL_INST_PATH)
            .join("intel-ucode.img")
            .exists()
        {
            ucode = format!("initrd /{}intel-ucode.img\n", REL_INST_PATH);
        }
        let initramfs = format!(
            "initrd /{}initramfs-{}-{}.img\n",
            REL_INST_PATH, self.distro, self.flavor
        );
        let options = format!("options {}", bootarg);
        let content = title + &vmlinuz + &ucode + &initramfs + &options;
        fs::write(entry_path, content)?;

        Ok(())
    }
}

use crate::println_with_prefix;
use anyhow::{anyhow, Result};
use semver::Version;
use std::{fmt, fs, path::Path};

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

impl PartialEq for Kernel {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl Eq for Kernel {}

impl fmt::Display for Kernel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}", self.version, self.distro, self.flavor)
    }
}

impl Kernel {
    pub fn get_name(&self) -> String {
        format!("{}-{}-{}", self.version, self.distro, self.flavor)
    }
    /// Install a specific kernel to the esp using the given kernel filename
    pub fn install(&self, install_path: &Path) -> Result<()> {
        // if the path does not exist, ask the user for initializing friend
        if !install_path.exists() {
            println_with_prefix!("{} does not exist. Doing nothing.", install_path.display());
            println_with_prefix!("If you wish to use systemd-boot, run systemd-boot-friend init.");
            println_with_prefix!("Or, if your ESP mountpoint is not at ESP_MOUNTPOINT, please edit /etc/systemd-boot-friend-rs.conf.");

            return Err(anyhow!("{} not found", install_path.display()));
        }
        // generate the path to the source files
        println_with_prefix!(
            "Installing {} to {} ...",
            self.get_name(),
            install_path.display()
        );
        let vmlinuz_path = format!(
            "/boot/vmlinuz-{}-{}-{}",
            self.version, self.distro, self.flavor
        );
        let initramfs_path = format!(
            "/boot/initramfs-{}-{}-{}.img",
            self.version, self.distro, self.flavor
        );
        let src_vmlinuz = Path::new(&vmlinuz_path);
        let src_initramfs = Path::new(&initramfs_path);
        let src_ucode = Path::new("/boot/intel-ucode.img");
        // Copy the source files to the `install_path` using specific
        // filename format, remove the version parts of the files
        if src_vmlinuz.exists() {
            fs::copy(
                &src_vmlinuz,
                install_path.join(format!("vmlinuz-{}-{}", self.distro, self.flavor)),
            )?;
        } else {
            return Err(anyhow!("Kernel file not found"));
        }

        if src_initramfs.exists() {
            fs::copy(
                &src_initramfs,
                install_path.join(format!("initramfs-{}-{}.img", self.distro, self.flavor)),
            )?;
        } else {
            return Err(anyhow!("Initramfs not found"));
        }

        // copy Intel ucode if exists
        if src_ucode.exists() {
            println_with_prefix!("intel-ucode detected. Installing ...");
            fs::copy(&src_ucode, install_path.join("intel-ucode.img"))?;
        }

        Ok(())
    }
}

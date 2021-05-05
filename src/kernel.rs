use crate::println_with_prefix;
use anyhow::{anyhow, Result};
use semver::Version;
use std::{fmt, fs, io::Write, path::Path};

const REL_INST_PATH: &str = "EFI/aosc/";
const SRC_PATH: &str = "/boot/";
const UCODE_PATH: &str = "/boot/intel-ucode.img";

/// A kernel struct for parsing kernel filenames
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
    /// Get the full name of the kernel
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

    /// Create a systemd-boot entry config
    pub fn make_config(&self, esp_path: &Path, bootarg: &str, force_write: bool) -> Result<()> {
        let entry_path = esp_path.join(format!(
            "loader/entries/{}-{}.conf",
            self.distro, self.flavor
        ));
        // do not override existed entry file until forced to do so
        if entry_path.exists() && !force_write {
            println_with_prefix!(
                "{} already exists. Doing nothing on this file.",
                entry_path.display()
            );
            println_with_prefix!("If you wish to override the file, specify -f and run again.");
            return Ok(());
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
        let mut ucode = "".to_string();
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

        let mut entry = fs::File::create(entry_path)?;
        entry.write_all(&content.as_bytes())?;

        Ok(())
    }
}

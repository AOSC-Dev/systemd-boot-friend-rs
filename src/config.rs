use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, fs, path::PathBuf, rc::Rc};

use crate::{fl, println_with_prefix, println_with_prefix_and_fl};

const CONF_PATH: &str = "/etc/systemd-boot-friend.conf";
const MOUNTS: &str = "/proc/mounts";
// const CMDLINE: &str = "/proc/cmdline";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(alias = "VMLINUX", alias = "VMLINUZ")]
    pub vmlinux: String,
    #[serde(alias = "INITRD")]
    pub initrd: String,
    #[serde(alias = "DISTRO")]
    pub distro: Rc<String>,
    #[serde(alias = "ESP_MOUNTPOINT")]
    pub esp_mountpoint: Rc<PathBuf>,
    #[serde(alias = "KEEP")]
    pub keep: Option<usize>,
    #[serde(alias = "BOOTARG")]
    bootarg: Option<String>, // for compatibility
    #[serde(alias = "BOOTARGS", default)]
    pub bootargs: Rc<RefCell<HashMap<String, String>>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            vmlinux: "vmlinuz-{VERSION}".to_owned(),
            initrd: "initramfs-{VERSION}.img".to_owned(),
            distro: Rc::new("Linux".to_owned()),
            esp_mountpoint: Rc::new(PathBuf::from("/efi")),
            keep: None,
            bootarg: None,
            bootargs: Rc::new(RefCell::new(HashMap::from([(
                "default".to_owned(),
                String::new(),
            )]))),
        }
    }
}

/// Detect current root partition, used for generating kernel cmdline
fn detect_root_partition() -> Result<String> {
    let mounts = fs::read_to_string(MOUNTS)?;
    let mut root_partition = String::new();

    for line in mounts.lines() {
        let mut parts = line.split_whitespace();
        let partition = parts.next().unwrap_or_default();
        let mount = parts.next().unwrap_or_default();
        if mount == "/" {
            partition.clone_into(&mut root_partition);
        }
    }

    Ok(root_partition)
}

/// Fill the necessary root cmdline and rw cmdline params if they are missing
fn fill_necessary_bootarg(bootarg: &str) -> Result<String> {
    let mut has_root = false;
    let mut has_rw = false;

    for param in bootarg.split_whitespace() {
        if param.starts_with("root=") {
            has_root = true;
        } else if param == "rw" || param == "ro" {
            has_rw = true;
        }
    }

    let mut filled_bootarg = String::from(bootarg.strip_suffix('\n').unwrap_or(bootarg));

    if !has_root {
        filled_bootarg.push_str(" root=");
        filled_bootarg.push_str(&detect_root_partition()?)
    }

    if !has_rw {
        filled_bootarg.push_str(" rw");
    }

    Ok(filled_bootarg)
}

impl Config {
    /// Write the current state to the configuration file
    fn write(&self) -> Result<()> {
        fs::create_dir_all(PathBuf::from(CONF_PATH).parent().unwrap())?;
        fs::write(CONF_PATH, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Read the configuration file
    pub fn read() -> Result<Self> {
        match fs::read_to_string(CONF_PATH) {
            Ok(f) => {
                let mut config: Config = toml::from_str(&f)?;

                // Migrate from old configuration
                let old_conf = "{VERSION}-{LOCALVERSION}";
                let new_conf = "{VERSION}";

                if config.vmlinux.contains(old_conf) || config.initrd.contains(old_conf) {
                    println_with_prefix_and_fl!("conf_old");
                    config.vmlinux = config.vmlinux.replace(old_conf, new_conf);
                    config.initrd = config.initrd.replace(old_conf, new_conf);
                    config.write()?;
                }

                // For compatibility
                if let Some(b) = config.bootarg {
                    config.bootargs.borrow_mut().insert("default".to_owned(), b);
                    config.bootarg = None;
                    config.write()?;
                }

                if config.bootargs.borrow().is_empty()
                    || config.bootargs.borrow().get("default").is_none()
                {
                    config
                        .bootargs
                        .borrow_mut()
                        .insert("default".to_owned(), String::new());
                    config.write()?;
                }

                for (_, bootarg) in config.bootargs.borrow_mut().iter_mut() {
                    fill_necessary_bootarg(bootarg)?.trim().clone_into(bootarg);
                }

                Ok(config)
            }
            Err(_) => {
                println_with_prefix_and_fl!("conf_default", conf_path = CONF_PATH);
                Config::default().write()?;
                Err(anyhow!(fl!("edit_conf", conf_path = CONF_PATH)))
            }
        }
    }

    // /// Try to fill an empty BOOTARG option in Config
    // fn fill_empty_bootargs(&mut self) -> Result<()> {
    //     print_block_with_fl!("notice_empty_bootarg");
    //
    //     if !Confirm::with_theme(&ColorfulTheme::default())
    //         .with_prompt(fl!("ask_empty_bootarg"))
    //         .default(true)
    //         .interact()?
    //     {
    //         return Ok(());
    //     }
    //
    //     let current_bootarg = String::from_utf8(fs::read(CMDLINE)?)?;
    //
    //     print_block_with_fl!("current_bootarg");
    //
    //     // print bootarg (kernel command line), wrap at col 80
    //     for line in wrap(
    //         &current_bootarg,
    //         Options::new(80)
    //             .word_separator(WordSeparator::AsciiSpace)
    //             .word_splitter(WordSplitter::NoHyphenation),
    //     ) {
    //         eprintln!("{}", style(line).bold());
    //     }
    //
    //     if Confirm::with_theme(&ColorfulTheme::default())
    //         .with_prompt(fl!("ask_current_bootarg"))
    //         .default(true)
    //         .interact()?
    //     {
    //         self.bootargs
    //             .borrow_mut()
    //             .insert("default".to_owned(), current_bootarg);
    //     } else {
    //         let root = detect_root_partition()?;
    //
    //         print_block_with_fl!("current_root", root = root.as_str());
    //
    //         if !Confirm::with_theme(&ColorfulTheme::default())
    //             .with_prompt(fl!("ask_current_root", root = root.as_str()))
    //             .default(true)
    //             .interact()?
    //         {
    //             bail!(fl!("edit_bootarg", config = CONF_PATH));
    //         }
    //
    //         self.bootargs
    //             .borrow_mut()
    //             .insert("default".to_owned(), format!("root={} rw", root));
    //     }
    //
    //     self.write()?;
    //
    //     Ok(())
    // }
}

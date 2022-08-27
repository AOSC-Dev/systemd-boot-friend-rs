use anyhow::{anyhow, bail, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, fs, path::PathBuf, rc::Rc};
use textwrap::{wrap, Options, WordSeparator, WordSplitter};

use crate::{fl, print_block_with_fl, println_with_prefix, println_with_prefix_and_fl};

const CONF_PATH: &str = "/etc/systemd-boot-friend.conf";
const MOUNTS: &str = "/proc/mounts";
const CMDLINE: &str = "/proc/cmdline";

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
            bootarg: None,
            bootargs: Rc::new(RefCell::new(HashMap::new())),
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
            root_partition = partition.to_owned();
        }
    }

    Ok(root_partition)
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
        match fs::read(CONF_PATH) {
            Ok(f) => {
                let mut config: Config = toml::from_slice(&f)?;

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

                if config.bootargs.borrow().is_empty() {
                    config.fill_empty_bootargs()?;
                } else if config.bootargs.borrow().get("default").is_none() {
                    return Err(anyhow!(fl!("require_default", conf_path = CONF_PATH)));
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

    /// Try to fill an empty BOOTARG option in Config
    fn fill_empty_bootargs(&mut self) -> Result<()> {
        print_block_with_fl!("notice_empty_bootarg");

        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(fl!("ask_empty_bootarg"))
            .default(true)
            .interact()?
        {
            return Ok(());
        }

        let current_bootarg = String::from_utf8(fs::read(CMDLINE)?)?;

        print_block_with_fl!("current_bootarg");

        // print bootarg (kernel command line), wrap at col 80
        for line in wrap(
            &current_bootarg,
            Options::new(80)
                .word_separator(WordSeparator::AsciiSpace)
                .word_splitter(WordSplitter::NoHyphenation),
        ) {
            eprintln!("{}", style(line).bold());
        }

        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(fl!("ask_current_bootarg"))
            .default(true)
            .interact()?
        {
            self.bootargs
                .borrow_mut()
                .insert("default".to_owned(), current_bootarg);
        } else {
            let root = detect_root_partition()?;

            print_block_with_fl!("current_root", root = root.as_str());

            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(fl!("ask_current_root", root = root.as_str()))
                .default(true)
                .interact()?
            {
                bail!(fl!("edit_bootarg", config = CONF_PATH));
            }

            self.bootargs
                .borrow_mut()
                .insert("default".to_owned(), format!("root={} rw", root));
        }

        self.write()?;

        Ok(())
    }
}

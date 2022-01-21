use anyhow::{anyhow, bail, Result};
use clap::Parser;
use cli::{Opts, SubCommands};
use core::default::Default;
use dialoguer::{theme::ColorfulTheme, Select};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use i18n::I18N_LOADER;
use kernel::Kernel;

mod cli;
mod i18n;
mod kernel;
mod macros;
mod parser;

const CONF_PATH: &str = "/etc/systemd-boot-friend.conf";
const REL_DEST_PATH: &str = "EFI/systemd-boot-friend/";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "VMLINUZ")]
    vmlinuz: String,
    #[serde(rename = "INITRD")]
    initrd: String,
    #[serde(rename = "DISTRO")]
    distro: String,
    #[serde(rename = "ESP_MOUNTPOINT")]
    esp_mountpoint: PathBuf,
    #[serde(rename = "BOOTARG")]
    bootarg: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            vmlinuz: "vmlinuz-{VERSION}".to_owned(),
            initrd: "initramfs-{VERSION}.img".to_owned(),
            distro: "Linux".to_owned(),
            esp_mountpoint: PathBuf::from("/efi"),
            bootarg: String::new(),
        }
    }
}

/// Choose a kernel using dialoguer
fn choose_kernel(kernels: &[Kernel]) -> Result<Kernel> {
    if kernels.is_empty() {
        bail!(fl!("empty_list"));
    }
    // build dialoguer Select for kernel selection
    let n = Select::with_theme(&ColorfulTheme::default())
        .items(kernels)
        .default(0)
        .interact()?;

    Ok(kernels[n].clone())
}

/// Update systemd-boot kernels and entries
fn update(installed_kernels: &[Kernel], kernels: &[Kernel]) -> Result<()> {
    for k in installed_kernels.iter() {
        k.remove()?;
    }
    for k in kernels.iter() {
        k.install_and_make_config(true)?;
    }

    Ok(())
}

/// Initialize the default environment for friend
fn init(config: &Config, installed_kernels: &[Kernel], kernels: &[Kernel]) -> Result<()> {
    // use bootctl to install systemd-boot
    println_with_prefix_and_fl!("initialize");
    Command::new("bootctl")
        .arg("install")
        .arg(
            "--esp=".to_owned()
                + config
                    .esp_mountpoint
                    .to_str()
                    .ok_or_else(|| anyhow!(fl!("invalid_esp")))?,
        )
        .stderr(Stdio::null())
        .spawn()?;
    // create folder structure
    println_with_prefix_and_fl!("create_folder");
    fs::create_dir_all(config.esp_mountpoint.join(REL_DEST_PATH))?;
    // Update systemd-boot kernels and entries
    update(installed_kernels, kernels)
}

#[inline]
fn parse_num_or_filename(config: &Config, n: &str, kernels: &[Kernel]) -> Result<Kernel> {
    Ok(match n.parse::<usize>() {
        Ok(num) => kernels
            .get(num - 1)
            .ok_or_else(|| anyhow!(fl!("invalid_index")))?
            .clone(),
        Err(_) => Kernel::parse(config, n)?,
    })
}

fn read_config() -> Result<Config> {
    fs::read(CONF_PATH).map_or_else(
        |_| {
            println_with_prefix_and_fl!("conf_default", conf_path = CONF_PATH);
            fs::create_dir_all(PathBuf::from(CONF_PATH).parent().unwrap())?;
            fs::write(CONF_PATH, toml::to_string_pretty(&Config::default())?)?;
            bail!(fl!("edit_conf", conf_path = CONF_PATH))
        },
        |f| {
            let mut config: Config = toml::from_slice(&f)?;
            // Migrate from old configuration
            let old_conf = "{VERSION}-{LOCALVERSION}";
            let new_conf = "{VERSION}";
            if config.vmlinuz.contains(old_conf) || config.initrd.contains(old_conf) {
                println_with_prefix_and_fl!("conf_old");
                config.vmlinuz = config.vmlinuz.replace(old_conf, new_conf);
                config.initrd = config.initrd.replace(old_conf, new_conf);
                fs::write(CONF_PATH, toml::to_string_pretty(&config)?)?;
            }
            Ok(config)
        },
    )
}

fn main() -> Result<()> {
    // CLI
    let matches: Opts = Opts::parse();
    // Read config, create a default one if the file is missing
    let config = read_config()?;
    let installed_kernels = Kernel::list_installed(&config)?;
    let kernels = Kernel::list(&config)?;
    // Switch table
    match matches.subcommands {
        Some(s) => match s {
            SubCommands::Init(_) => init(&config, &installed_kernels, &kernels)?,
            SubCommands::List(_) => {
                // list available kernels
                for (i, k) in kernels.iter().enumerate() {
                    println!("\u{001b}[1m[{}]\u{001b}[0m {}", i + 1, k);
                }
            }
            SubCommands::Install(args) => {
                let kernel = match args.target {
                    // the target can be both the number in
                    // the list and the name of the kernel
                    Some(n) => parse_num_or_filename(&config, &n, &kernels)?,
                    // install the newest kernel
                    // when no target is given
                    None => kernels
                        .first()
                        .ok_or_else(|| anyhow!(fl!("no_kernel")))?
                        .clone(),
                };
                kernel.install_and_make_config(args.force)?;
            }
            SubCommands::ListInstalled(_) => {
                for (i, k) in installed_kernels.iter().enumerate() {
                    println!("\u{001b}[1m[{}]\u{001b}[0m {}", i + 1, k);
                }
            }
            SubCommands::Remove(args) => {
                let kernel = match args.target {
                    // the target can be both the number in
                    // the list and the name of the kernel
                    Some(n) => parse_num_or_filename(&config, &n, &installed_kernels)?,
                    // select the kernel to remove
                    // when no target is given
                    None => choose_kernel(&installed_kernels)?,
                };
                kernel.remove()?;
            }
            SubCommands::Update(_) => update(&installed_kernels, &kernels)?,
        },
        None => choose_kernel(&kernels)?.install_and_make_config(false)?,
    }

    Ok(())
}

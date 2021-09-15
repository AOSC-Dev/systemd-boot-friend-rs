use anyhow::{anyhow, Result};
use argh::from_env;
use cli::{Interface, SubCommandEnum};
use core::default::Default;
use dialoguer::{theme::ColorfulTheme, Select};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use i18n::I18N_LOADER;
use kernel::Kernel;

mod cli;
mod i18n;
mod kernel;
mod macros;

const CONF_PATH: &str = "/etc/systemd-boot-friend.conf";
const REL_DEST_PATH: &str = "EFI/systemd-boot-friend/";
const INSTALLED_PATH: &str = "/var/lib/systemd-boot-friend/installed.json";

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
            vmlinuz: "vmlinuz-{VERSION}-{LOCALVERSION}".to_owned(),
            initrd: "initramfs-{VERSION}-{LOCALVERSION}.img".to_owned(),
            distro: "AOSC OS".to_owned(),
            esp_mountpoint: PathBuf::from("/efi"),
            bootarg: String::new(),
        }
    }
}

/// Read config, create a default one if the file is missing
fn read_config() -> Result<Config> {
    fs::read(CONF_PATH).map_or_else(
        |_| {
            println_with_prefix_and_fl!("conf_default", conf_path = CONF_PATH);
            fs::create_dir_all(PathBuf::from(CONF_PATH).parent().unwrap())?;
            fs::write(CONF_PATH, toml::to_string_pretty(&Config::default())?)?;
            Err(anyhow!(fl!("edit_conf", conf_path = CONF_PATH)))
        },
        |f| Ok(toml::from_slice(&f)?),
    )
}

/// Choose a kernel using dialoguer
fn choose_kernel(kernels: &[Kernel]) -> Result<Kernel> {
    if kernels.is_empty() {
        return Err(anyhow!(fl!("empty_list")));
    }
    // build dialoguer Select for kernel selection
    let n = Select::with_theme(&ColorfulTheme::default())
        .items(kernels)
        .default(0)
        .interact()?;

    Ok(kernels[n].clone())
}

/// Update systemd-boot kernels and entries
fn update(config: &Config, installed_kernels: &mut Vec<Kernel>, kernels: &[Kernel]) -> Result<()> {
    while let Some(k) = installed_kernels.pop() {
        k.remove(config)?;
    }
    for k in kernels.iter() {
        k.install_and_make_config(config, true)?;
        installed_kernels.push(k.clone());
    }

    Ok(())
}

/// Initialize the default environment for friend
fn init(config: &Config, installed_kernels: &mut Vec<Kernel>, kernels: &[Kernel]) -> Result<()> {
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
    update(config, installed_kernels, kernels)
}

#[inline]
fn parse_num_or_filename(config: &Config, n: &str, kernels: &[Kernel]) -> Result<Kernel> {
    Ok(match n.parse::<usize>() {
        Ok(num) => kernels
            .get(num - 1)
            .ok_or_else(|| anyhow!(fl!("invalid_index")))?
            .clone(),
        Err(_) => Kernel::parse(n, config)?,
    })
}

fn main() -> Result<()> {
    // CLI
    let matches: Interface = from_env();
    if matches.version {
        println_with_prefix!(env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    // Read config
    let config = read_config()?;
    // the record file of installed kernels, use empty value if not found
    let mut installed_kernels = Vec::new();
    if let Ok(f) = fs::read(INSTALLED_PATH) {
        installed_kernels = serde_json::from_slice::<Vec<String>>(&f)?
            .iter()
            .map(|s| Kernel::parse(s, &config))
            .collect::<Result<Vec<Kernel>>>()?;
    } else {
        // Create the folder structure for the record of installed kernels
        fs::create_dir_all(Path::new(INSTALLED_PATH).parent().unwrap())?;
        serde_json::to_writer(fs::File::create(INSTALLED_PATH)?, &Vec::<String>::new())?;
    }
    let kernels = Kernel::list_kernels(&config)?;
    // Switch table
    match matches.nested {
        Some(s) => match s {
            SubCommandEnum::Init(_) => init(&config, &mut installed_kernels, &kernels)?,
            SubCommandEnum::List(_) => {
                // list available kernels
                for (i, k) in Kernel::list_kernels(&config)?.iter().enumerate() {
                    println!("\u{001b}[1m[{}]\u{001b}[0m {}", i + 1, k);
                }
            }
            SubCommandEnum::Install(args) => {
                let kernel = match args.target {
                    // the target can be both the number in
                    // the list and the name of the kernel
                    Some(n) => parse_num_or_filename(&config, &n, &kernels)?,
                    // install the newest kernel
                    // when no target is given
                    None => Kernel::list_kernels(&config)?
                        .first()
                        .ok_or_else(|| anyhow!(fl!("no_kernel")))?
                        .clone(),
                };
                kernel.install_and_make_config(&config, args.force)?;
                installed_kernels.push(kernel);
            }
            SubCommandEnum::ListInstalled(_) => {
                for (i, k) in installed_kernels.iter().enumerate() {
                    println!("\u{001b}[1m[{}]\u{001b}[0m {}", i + 1, k);
                }
            }
            SubCommandEnum::Remove(args) => {
                let kernel = match args.target {
                    // the target can be both the number in
                    // the list and the name of the kernel
                    Some(n) => parse_num_or_filename(&config, &n, &installed_kernels)?,
                    // select the kernel to remove
                    // when no target is given
                    None => choose_kernel(&installed_kernels)?,
                };
                kernel.remove(&config)?;
                installed_kernels.retain(|k| *k != kernel);
            }
            SubCommandEnum::Update(_) => update(
                &config,
                &mut installed_kernels,
                &Kernel::list_kernels(&config)?,
            )?,
        },
        None => {
            let kernel = choose_kernel(&Kernel::list_kernels(&config)?)?;
            // make sure the kernel selected is installed
            kernel.install_and_make_config(&config, false)?;
            installed_kernels.push(kernel);
        }
    }
    // Write the installed kernels file
    installed_kernels.sort();
    installed_kernels.dedup();
    let installed_kernels: Vec<String> =
        installed_kernels.iter().map(|k| format!("{}", k)).collect();
    serde_json::to_writer(fs::File::create(INSTALLED_PATH)?, &installed_kernels)?;

    Ok(())
}

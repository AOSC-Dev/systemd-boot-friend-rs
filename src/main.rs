use anyhow::{anyhow, Result};
use argh::from_env;
use cli::{Interface, SubCommandEnum};
use core::default::Default;
use dialoguer::{theme::ColorfulTheme, Select};
use kernel::Kernel;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

mod cli;
mod kernel;
mod macros;

const CONF_PATH: &str = "/etc/systemd-boot-friend.conf";
const REL_DEST_PATH: &str = "EFI/systemd-boot-friend/";
const INSTALLED_PATH: &str = "/var/lib/systemd-boot-friend/installed.json";

#[derive(Debug, Deserialize)]
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

/// Choose a kernel using dialoguer
fn choose_kernel(kernels: &[Kernel]) -> Result<Kernel> {
    if kernels.is_empty() {
        return Err(anyhow!("Empty list"));
    }
    // build dialoguer Select for kernel selection
    let n = Select::with_theme(&ColorfulTheme::default())
        .items(kernels)
        .default(0)
        .interact()?;

    Ok(kernels[n].clone())
}

/// Initialize the default environment for friend
fn init(config: &Config) -> Result<Kernel> {
    // use bootctl to install systemd-boot
    println_with_prefix!("Initializing systemd-boot ...");
    Command::new("bootctl")
        .arg("install")
        .arg(
            "--esp=".to_owned()
                + config
                    .esp_mountpoint
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid ESP_MOUNTPOINT"))?,
        )
        .stderr(Stdio::null())
        .spawn()?;
    // create folder structure
    println_with_prefix!("Creating folder structure for friend ...");
    fs::create_dir_all(config.esp_mountpoint.join(REL_DEST_PATH))?;
    // choose the kernel to install and
    // write the entry config file
    let kernel = choose_kernel(&Kernel::list_kernels(config)?)?;
    kernel.install_and_make_config(config, false)?;

    Ok(kernel)
}

fn main() -> Result<()> {
    // CLI
    let matches: Interface = from_env();
    if matches.version {
        println_with_prefix!(env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    // Read config
    let config: Config = toml::from_slice(&fs::read(CONF_PATH)?)?;
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
    // Switch table
    match matches.nested {
        Some(s) => match s {
            SubCommandEnum::Init(_) => {
                installed_kernels.push(init(&config)?);
            }
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
                    Some(n) => match n.parse::<usize>() {
                        Ok(num) => Kernel::list_kernels(&config)?
                            .get(num - 1)
                            .ok_or_else(|| anyhow!("Invalid kernel number"))?
                            .clone(),
                        Err(_) => Kernel::parse(&n, &config)?,
                    },
                    // install the newest kernel
                    // when no target is given
                    None => Kernel::list_kernels(&config)?
                        .first()
                        .ok_or_else(|| anyhow!("No kernel found"))?
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
                    Some(n) => match n.parse::<usize>() {
                        Ok(num) => installed_kernels
                            .get(num - 1)
                            .ok_or_else(|| anyhow!("Invalid kernel number"))?
                            .clone(),
                        Err(_) => Kernel::parse(&n, &config)?,
                    },
                    // select the kernel to remove
                    // when no target is given
                    None => choose_kernel(&installed_kernels)?,
                };
                kernel.remove(&config)?;
                installed_kernels.retain(|k| *k != kernel);
            }
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

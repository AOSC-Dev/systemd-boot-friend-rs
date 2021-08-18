use anyhow::{anyhow, Result};
use argh::from_env;
use cli::{Interface, SubCommandEnum};
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
const REL_INST_PATH: &str = "EFI/aosc/";

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(rename = "DISTRO")]
    distro: String,
    #[serde(rename = "ESP_MOUNTPOINT")]
    esp_mountpoint: PathBuf,
    #[serde(rename = "BOOTARG")]
    bootarg: String,
}

/// Choose a kernel using dialoguer
fn choose_kernel() -> Result<Kernel> {
    let kernels = Kernel::list_kernels()?;
    // build dialoguer Select for kernel selection
    let n = Select::with_theme(&ColorfulTheme::default())
        .items(&kernels)
        .default(0)
        .interact()?;

    Ok(kernels[n].clone())
}

/// Initialize the default environment for friend
fn init(distro: &str, esp_path: &Path, bootarg: &str) -> Result<()> {
    // use bootctl to install systemd-boot
    println_with_prefix!("Initializing systemd-boot ...");
    Command::new("bootctl")
        .arg("install")
        .arg(
            "--esp=".to_owned()
                + esp_path
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid ESP_MOUNTPOINT"))?,
        )
        .stderr(Stdio::null())
        .spawn()?;
    // create folder structure
    println_with_prefix!("Creating folder structure for friend ...");
    fs::create_dir_all(esp_path.join(REL_INST_PATH))?;
    // choose the kernel to install and
    // write the entry config file
    let kernel = choose_kernel()?;
    kernel.install(esp_path)?;
    kernel.make_config(distro, esp_path, bootarg, false)?;

    Ok(())
}

fn main() -> Result<()> {
    // Read config
    let config: Config = toml::from_slice(&fs::read(CONF_PATH)?)?;
    // CLI
    let matches: Interface = from_env();
    if matches.version {
        println_with_prefix!(env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    // Switch table
    match matches.nested {
        Some(s) => match s {
            SubCommandEnum::Init(_) => {
                init(&config.distro, &config.esp_mountpoint, &config.bootarg)?
            }
            SubCommandEnum::List(_) => {
                // list available kernels
                for (i, k) in Kernel::list_kernels()?.iter().enumerate() {
                    println!("\u{001b}[1m[{}]\u{001b}[0m {}", i + 1, k);
                }
            }
            SubCommandEnum::Install(args) => {
                match args.target {
                    // the target can be both the number in
                    // the list and the name of the kernel
                    Some(n) => {
                        let kernel = match n.parse::<usize>() {
                            Ok(num) => Kernel::list_kernels()?[num - 1].clone(),
                            Err(_) => Kernel::parse(&n)?,
                        };
                        kernel.install_and_make_config(
                            &config.distro,
                            &config.esp_mountpoint,
                            &config.bootarg,
                            args.force,
                        )?;
                    }
                    // installs the newest kernel
                    // when no target is given
                    None => {
                        let kernel = &Kernel::list_kernels()?[0];
                        kernel.install_and_make_config(
                            &config.distro,
                            &config.esp_mountpoint,
                            &config.bootarg,
                            args.force,
                        )?
                    }
                }
            }
        },
        None => {
            let kernel = choose_kernel()?;
            // make sure the kernel selected is installed
            kernel.install_and_make_config(
                &config.distro,
                &config.esp_mountpoint,
                &config.bootarg,
                false,
            )?;
        }
    }

    Ok(())
}

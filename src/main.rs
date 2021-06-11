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
const MODULES_PATH: &str = "/usr/lib/modules/";

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(rename = "ESP_MOUNTPOINT")]
    esp_mountpoint: PathBuf,
    #[serde(rename = "BOOTARG")]
    bootarg: String,
}

/// Generate a sorted vector of kernel filenames
fn list_kernels() -> Result<Vec<Kernel>> {
    // read /usr/lib/modules to get kernel filenames
    let mut kernels = fs::read_dir(MODULES_PATH)?
        .map(|k| {
            Kernel::parse(
                &k?.file_name()
                    .into_string()
                    .unwrap_or_else(|_| String::new()),
            )
        })
        .collect::<Result<Vec<Kernel>>>()?;

    // Sort the vector, thus the kernel filenames are
    // arranged with versions from newer to older
    kernels.sort_by(|a, b| b.cmp(a));
    Ok(kernels)
}

/// Choose a kernel using dialoguer
fn choose_kernel() -> Result<Kernel> {
    let kernels = list_kernels()?;
    // build dialoguer Select for kernel selection
    let n = Select::with_theme(&ColorfulTheme::default())
        .items(&kernels)
        .default(0)
        .interact()?;

    Ok(kernels[n].clone())
}

/// Initialize the default environment for friend
fn init(esp_path: &Path, bootarg: &str) -> Result<()> {
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
    kernel.make_config(esp_path, bootarg, false)?;

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
            SubCommandEnum::Init(_) => init(&config.esp_mountpoint, &config.bootarg)?,
            SubCommandEnum::MakeConf(args) => {
                let kernel = choose_kernel()?;
                // make sure the kernel selected is installed
                kernel.install(&config.esp_mountpoint)?;
                kernel.make_config(&config.esp_mountpoint, &config.bootarg, args.force)?;
            }
            SubCommandEnum::List(_) => {
                // list available kernels
                for (i, k) in list_kernels()?.iter().enumerate() {
                    println!("\u{001b}[1m[{}]\u{001b}[0m {}", i + 1, k);
                }
            }
            SubCommandEnum::Install(args) => {
                if let Some(n) = args.target {
                    // the target can be both the number in
                    // the list and the name of the kernel
                    match n.parse::<usize>() {
                        Ok(num) => list_kernels()?[num - 1].install(&config.esp_mountpoint)?,
                        Err(_) => Kernel::parse(&n)?.install(&config.esp_mountpoint)?,
                    }
                } else {
                    // installs the newest kernel
                    // when no target is given
                    list_kernels()?[0].install(&config.esp_mountpoint)?;
                }
            }
        },
        None => choose_kernel()?.install(&config.esp_mountpoint)?,
    }

    Ok(())
}

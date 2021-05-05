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

/// Reads the configuration file at CONF_PATH
fn read_conf() -> Result<Config> {
    let content = fs::read(CONF_PATH)?;
    // deserialize into Config struct
    let config: Config = toml::from_slice(&content)?;

    Ok(config)
}

/// Initialize the default environment for friend
fn init(install_path: &Path, esp_path: &Path, bootarg: &str) -> Result<()> {
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
        .stdout(Stdio::null())
        .spawn()?;
    // create folder structure
    println_with_prefix!("Creating folder structure for friend ...");
    fs::create_dir_all(install_path)?;
    let newest_kernel = &list_kernels()?[0];
    // install the newest kernel
    newest_kernel.install(install_path)?;
    // Create systemd-boot entry config
    newest_kernel.make_config(esp_path, bootarg, true)?;

    Ok(())
}

/// Generate a sorted vector of kernel filenames
fn list_kernels() -> Result<Vec<Kernel>> {
    // read /usr/lib/modules to get kernel filenames
    let kernels = fs::read_dir(MODULES_PATH)?;
    let mut kernels_list = Vec::new();
    for kernel in kernels {
        let kernel_name = kernel.unwrap().file_name().into_string().unwrap();
        kernels_list.push(Kernel::parse(&kernel_name)?);
    }
    // Sort the vector, thus the kernel filenames are
    // arranged with versions from older to newer
    kernels_list.sort();
    kernels_list.reverse();

    Ok(kernels_list)
}

fn print_kernels() -> Result<()> {
    let kernels = list_kernels()?;
    // print kernel filenames with numbers for users to choose
    for (i, k) in kernels.iter().enumerate() {
        println!("[{}] {}", i + 1, k);
    }
    Ok(())
}

/// Default behavior when calling without any subcommands
fn ask_for_kernel(install_path: &Path) -> Result<()> {
    let kernels = list_kernels()?;
    // build dialoguer Select for kernel selection
    let theme = ColorfulTheme::default();
    let n = Select::with_theme(&theme)
        .items(&kernels)
        .default(0)
        .interact()?;

    kernels[n].install(install_path)?;

    Ok(())
}

/// Ask for the kernel to write the entry config
fn ask_for_config(
    install_path: &Path,
    esp_path: &Path,
    bootarg: &str,
    force_write: bool,
) -> Result<()> {
    let kernels = list_kernels()?;
    // build dialoguer Select for kernel selection
    let theme = ColorfulTheme::default();
    let n = Select::with_theme(&theme)
        .items(&kernels)
        .default(0)
        .interact()?;

    // make sure the kernel is present at REL_INST_PATH
    kernels[n].install(install_path)?;
    // generate the entry config
    kernels[n].make_config(esp_path, bootarg, force_write)?;

    Ok(())
}

fn main() -> Result<()> {
    let config = read_conf()?;
    let install_path = config.esp_mountpoint.join(REL_INST_PATH);
    let matches: Interface = from_env();
    if matches.version {
        println_with_prefix!(env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    // Switch table
    match matches.nested {
        Some(s) => match s {
            SubCommandEnum::Init(_) => {
                init(&install_path, &config.esp_mountpoint, &config.bootarg)?
            }
            SubCommandEnum::MakeConf(args) => ask_for_config(
                &install_path,
                &config.esp_mountpoint,
                &config.bootarg,
                args.force,
            )?,
            SubCommandEnum::List(_) => print_kernels()?,
            SubCommandEnum::InstallKernel(args) => {
                if let Some(n) = args.target {
                    match n.parse::<usize>() {
                        Ok(num) => list_kernels()?[num].install(&install_path)?,
                        Err(_) => Kernel::parse(&n)?.install(&install_path)?,
                    }
                } else {
                    list_kernels()?[0].install(&install_path)?
                }
            }
        },
        None => ask_for_kernel(&install_path)?,
    }

    Ok(())
}

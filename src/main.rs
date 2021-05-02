#![allow(non_snake_case)]

use anyhow::{anyhow, Result};
use argh::from_env;
use cli::{Interface, SubCommandEnum};
use dialoguer::{theme::ColorfulTheme, Select};
use serde::Deserialize;
use semver::Version;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

mod cli;

const CONF_PATH: &str = "/etc/systemd-boot-friend.conf";
const REL_INST_PATH: &str = "EFI/aosc/";
const OUTPUT_PREFIX: &str = "\u{001b}[1m[systemd-boot-friend]\u{001b}[0m";

#[derive(Debug, Deserialize)]
struct Config {
    ESP_MOUNTPOINT: String,
    // BOOTARG: String, // not implemented yet
}

macro_rules! println_with_prefix {
    ($($arg:tt)+) => {
        print!("{} ", OUTPUT_PREFIX);
        println!($($arg)+);
    };
}

macro_rules! yield_into {
    { ( $x:ident ) = $v:expr, $e:expr } => {
        $x = $v.next().ok_or_else(|| anyhow!("{}", $e))?;
    };
    { ( $x:ident, $($y:ident),+ ) = $v:expr, $e:expr } => {
        $x = $v.next().ok_or_else(|| anyhow!("{}", $e))?;
        yield_into!(($($y),+) = $v, $e);
    }
}

/// Reads the configuration file at CONF_PATH
fn read_conf() -> Result<Config> {
    let content = fs::read(CONF_PATH)?;
    // deserialize into Config struct
    let config: Config = toml::from_slice(&content)?;

    Ok(config)
}

/// Initialize the default environment for friend
fn init(install_path: &Path, esp_path: &str) -> Result<()> {
    // use bootctl to install systemd-boot
    println_with_prefix!("Initializing systemd-boot ...");
    Command::new("bootctl")
        .arg("install")
        .arg("--esp=".to_owned() + esp_path)
        .stdout(Stdio::null())
        .spawn()?;
    // create folder structure
    println_with_prefix!("Creating folder structure for friend ...");
    fs::create_dir_all(install_path)?;
    // install the newest kernel
    install_newest_kernel(install_path)?;

    // Currently users have to manually create the boot entry config,
    // boot entry auto generator may be implemented in the future
    println!("Please make sure you have written the boot entry config for systemd-boot,");
    println!("see https://systemd.io/BOOT_LOADER_SPECIFICATION/ for further information.");

    Ok(())
}

/// Generate a sorted vector of kernel filenames
fn list_kernels() -> Result<Vec<Version>> {
    // read /usr/lib/modules to get kernel filenames
    let kernels = fs::read_dir("/usr/lib/modules")?;
    let mut kernels_list = Vec::new();
    for kernel in kernels {
        kernels_list.push(Version::parse(&kernel.unwrap().file_name().into_string().unwrap())?);
    }
    // Sort the vector, thus the kernel filenames are
    // arranged with versions from older to newer
    kernels_list.sort();

    Ok(kernels_list)
}

fn print_kernels() -> Result<()> {
    let kernels = list_kernels()?;
    // print kernel filenames with numbers for users to choose
    for (i, k) in kernels.into_iter().enumerate() {
        println!("[{}] {}", i + 1, k);
    }

    Ok(())
}

/// Install a specific kernel to the esp using the given kernel filename
fn install_kernel(kernel_name: &str, install_path: &Path) -> Result<()> {
    // if the path does not exist, ask the user for initializing friend
    if !install_path.exists() {
        println!("{} does not exist. Doing nothing.", install_path.display());
        println!("If you wish to use systemd-boot, run systemd-boot-friend init.");
        println!("Or, if your ESP mountpoint is not at esp_mountpoint, please edit /etc/systemd-boot-friend-rs.conf.");

        return Err(anyhow!("{} not found", install_path.display()));
    }
    // Split the kernel filename into 3 parts in order to determine
    // the version, name and the flavor of the chosen kernel
    let mut splitted_kernel_name = kernel_name.splitn(3, '-');
    let kernel_version;
    let distro_name;
    let kernel_flavor;
    yield_into!(
        (kernel_version, distro_name, kernel_flavor) = splitted_kernel_name,
        "Invalid kernel filename"
    );
    // generate the path to the source files
    println_with_prefix!(
        "Installing {} to {} ...",
        kernel_name,
        install_path.display()
    );
    let vmlinuz_path = format!(
        "/boot/vmlinuz-{}-{}-{}",
        kernel_version, distro_name, kernel_flavor
    );
    let initramfs_path = format!(
        "/boot/initramfs-{}-{}-{}.img",
        kernel_version, distro_name, kernel_flavor
    );
    let src_vmlinuz = Path::new(&vmlinuz_path);
    let src_initramfs = Path::new(&initramfs_path);
    // Copy the source files to the `install_path` using specific
    // filename format, remove the version parts of the files
    if src_vmlinuz.exists() {
        fs::copy(
            &src_vmlinuz,
            &format!(
                "{}vmlinuz-{}-{}",
                install_path.display(),
                distro_name,
                kernel_flavor
            ),
        )?;
    } else {
        return Err(anyhow!("Kernel file not found"));
    }

    if src_initramfs.exists() {
        fs::copy(
            &src_initramfs,
            &format!(
                "{}initramfs-{}-{}.img",
                install_path.display(),
                distro_name,
                kernel_flavor
            ),
        )?;
    } else {
        return Err(anyhow!("Initramfs not found"));
    }

    Ok(())
}

/// Install a specific kernel to the esp using the given position in the kernel list
fn install_specific_kernel_in_list(install_path: &Path, n: usize) -> Result<()> {
    let kernels = list_kernels()?;
    if n >= kernels.len() {
        return Err(anyhow!("Chosen kernel index out of bound"));
    }
    install_kernel(&kernels[n].to_string(), install_path)?;

    Ok(())
}

fn install_newest_kernel(install_path: &Path) -> Result<()> {
    println_with_prefix!("Installing the newest kernel ...");
    let kernels = list_kernels()?;
    // Install the last one in the kernel list as the list
    // has already been sorted by filename and version
    install_kernel(&kernels[kernels.len() - 1].to_string(), install_path)?;

    Ok(())
}

/// Default behavior when calling without any subcommands
fn ask_for_kernel(install_path: &Path) -> Result<()> {
    let kernels = list_kernels()?;
    // build dialoguer Select for kernel selection
    let theme = ColorfulTheme::default();
    let n = Select::with_theme(&theme)
        .items(&kernels)
        .default(kernels.len() - 1)
        .interact()?;

    install_specific_kernel_in_list(install_path, n)?;

    Ok(())
}

fn main() -> Result<()> {
    let config = read_conf()?;
    let install_path = Path::new(&config.ESP_MOUNTPOINT).join(REL_INST_PATH);
    let matches: Interface = from_env();
    if matches.version {
        println_with_prefix!(env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    // Switch table
    match matches.nested {
        Some(s) => match s {
            SubCommandEnum::Init(_) => init(&install_path, &config.ESP_MOUNTPOINT)?,
            SubCommandEnum::List(_) => print_kernels()?,
            SubCommandEnum::InstallKernel(args) => {
                if let Some(n) = args.target {
                    match n.parse::<usize>() {
                        Ok(num) => install_specific_kernel_in_list(&install_path, num - 1)?,
                        Err(_) => install_kernel(&n, &install_path)?,
                    }
                } else {
                    install_newest_kernel(&install_path)?
                }
            }
        },
        None => ask_for_kernel(&install_path)?,
    }

    Ok(())
}

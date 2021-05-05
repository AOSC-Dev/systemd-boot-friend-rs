use anyhow::{anyhow, Result};
use argh::from_env;
use cli::{Interface, SubCommandEnum};
use dialoguer::{theme::ColorfulTheme, Select};
use kernel::Kernel;
use semver::Version;
use serde::Deserialize;
use std::{
    fs,
    io::Write,
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
    // install the newest kernel
    install_newest_kernel(install_path)?;

    // Create systemd-boot entry config
    make_config(esp_path, bootarg, true)?;

    Ok(())
}

/// Generate a sorted vector of kernel filenames
fn list_kernels() -> Result<Vec<Kernel>> {
    // read /usr/lib/modules to get kernel filenames
    let kernels = fs::read_dir(MODULES_PATH)?;
    let mut kernels_list = Vec::new();
    for kernel in kernels {
        let kernel_name = kernel.unwrap().file_name().into_string().unwrap();
        kernels_list.push(parse_kernel_name(&kernel_name)?);
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

fn parse_kernel_name(kernel_name: &str) -> Result<Kernel> {
    // Split the kernel filename into 3 parts in order to determine
    // the version, name and the flavor of the kernel
    let mut splitted_kernel_name = kernel_name.splitn(3, '-');
    let kernel_version;
    let distro_name;
    let kernel_flavor;
    yield_into!(
        (kernel_version, distro_name, kernel_flavor) = splitted_kernel_name,
        "Invalid kernel filename"
    );
    Ok(Kernel {
        version: Version::parse(kernel_version)?,
        distro: distro_name.to_string(),
        flavor: kernel_flavor.to_string(),
    })
}

/// Install a specific kernel to the esp using the given position in the kernel list
fn install_specific_kernel_in_list(install_path: &Path, n: usize) -> Result<()> {
    let kernels = list_kernels()?;
    if n >= kernels.len() {
        return Err(anyhow!("Chosen kernel index out of bound"));
    }
    kernels[n].install(install_path)?;

    Ok(())
}

fn install_newest_kernel(install_path: &Path) -> Result<()> {
    println_with_prefix!("Installing the newest kernel ...");
    // Install the last one in the kernel list as the list
    // has already been sorted by filename and version
    list_kernels()?[0].install(install_path)?;

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

    install_specific_kernel_in_list(install_path, n)?;

    Ok(())
}

/// Create a systemd-boot entry config
fn make_config(esp_path: &Path, bootarg: &str, force_write: bool) -> Result<()> {
    let newest_kernel = &list_kernels()?[0];
    let entry_path = esp_path.join(format!(
        "loader/entries/{}-{}.conf",
        newest_kernel.distro, newest_kernel.flavor
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
        newest_kernel,
        entry_path.display()
    );
    // Generate entry config
    let title = format!("title AOSC OS ({})\n", newest_kernel.flavor);
    let vmlinuz = format!(
        "linux /{}vmlinuz-{}-{}\n",
        REL_INST_PATH, newest_kernel.distro, newest_kernel.flavor
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
        REL_INST_PATH, newest_kernel.distro, newest_kernel.flavor
    );
    let options = format!("options {}", bootarg);
    let content = title + &vmlinuz + &ucode + &initramfs + &options;

    let mut entry = fs::File::create(entry_path)?;
    entry.write(&content.as_bytes())?;

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
            SubCommandEnum::MakeConf(args) => {
                make_config(&config.esp_mountpoint, &config.bootarg, args.force)?
            }
            SubCommandEnum::List(_) => print_kernels()?,
            SubCommandEnum::InstallKernel(args) => {
                if let Some(n) = args.target {
                    match n.parse::<usize>() {
                        Ok(num) => install_specific_kernel_in_list(&install_path, num - 1)?,
                        Err(_) => parse_kernel_name(&n)?.install(&install_path)?,
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

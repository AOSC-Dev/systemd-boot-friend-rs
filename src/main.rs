use anyhow::{anyhow, Result};
use clap::{crate_version, App, Arg, SubCommand};
use dialoguer::{theme::ColorfulTheme, Select};
use serde::Deserialize;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

const CONF_PATH: &str = "/etc/systemd-boot-friend-rs.conf";
const REL_INST_PATH: &str = "EFI/aosc/";

#[derive(Deserialize)]
struct Config {
    esp_mountpoint: String,
}

fn read_conf() -> Result<Config> {
    let content = fs::read(CONF_PATH)?;
    let config: Config = serde_json::from_slice(&content)?;

    Ok(config)
}

fn init(inst_path: &Path) -> Result<()> {
    println!("Initializing systemd-boot ...");
    Command::new("bootctl")
        .arg("install")
        .arg("--esp=".to_owned() + inst_path.to_str().unwrap_or("/efi"))
        .stdout(Stdio::null())
        .spawn()?;

    println!("Creating folder structure for friend ...");
    fs::create_dir_all(Path::new(inst_path).join(REL_INST_PATH))?;

    install_newest_kernel(&inst_path)?;

    println!("Please make sure you have written the boot entry config for systemd-boot,");
    println!("see https://systemd.io/BOOT_LOADER_SPECIFICATION/ for further information.");

    Ok(())
}

fn list_kernels() -> Result<Vec<String>> {
    let kernels = fs::read_dir("/usr/lib/modules")?;
    let mut kernels_ls = Vec::new();

    for kernel in kernels {
        kernels_ls.push(kernel.unwrap().file_name().into_string().unwrap());
    }
    kernels_ls.sort();

    Ok(kernels_ls)
}

fn print_kernels() -> Result<()> {
    let kernels = list_kernels()?;

    for (i, k) in kernels.into_iter().enumerate() {
        println!("[{}] {}", i + 1, k);
    }

    Ok(())
}

fn install_kernel(kernel_name: &str, inst_path: &Path) -> Result<()> {
    if !inst_path.exists() {
        println!("{} does not exist. Doing nothing.", inst_path.display());
        println!("If you wish to use systemd-boot, run systemd-boot-friend init.");
        println!("Or, if your ESP mountpoint is not at esp_mountpoint, please edit /etc/systemd-boot-friend-rs.conf.");

        return Err(anyhow!("{} not found", inst_path.display()));
    }

    println!("Installing {} to {} ...", kernel_name, inst_path.display());
    let mut splitted_kernel_name = kernel_name.splitn(3, '-');
    let kernel_version = splitted_kernel_name
        .next()
        .ok_or_else(|| anyhow!("Not standard kernel filename"))?;
    let distro_name = splitted_kernel_name
        .next()
        .ok_or_else(|| anyhow!("Not standard kernel filename"))?;
    let kernel_flavor = splitted_kernel_name
        .next()
        .ok_or_else(|| anyhow!("Not standard kernel filename"))?;

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

    if src_vmlinuz.exists() {
        fs::copy(
            &src_vmlinuz,
            &format!(
                "{}vmlinuz-{}-{}",
                inst_path.display(),
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
                inst_path.display(),
                distro_name,
                kernel_flavor
            ),
        )?;
    } else {
        return Err(anyhow!("Initramfs not found"));
    }

    Ok(())
}

fn install_spec_kernel(inst_path: &Path, n: usize) -> Result<()> {
    let kernels = list_kernels()?;
    if n >= kernels.len() {
        return Err(anyhow!("Chosen kernel index out of bound"));
    }
    install_kernel(&kernels[n], inst_path)?;

    Ok(())
}

fn install_newest_kernel(inst_path: &Path) -> Result<()> {
    println!("Installing the newest kernel ...");
    let kernels = list_kernels()?;
    install_kernel(&kernels[kernels.len() - 1], inst_path)?;

    Ok(())
}

fn ask_for_kernel(inst_path: &Path) -> Result<()> {
    let theme = ColorfulTheme::default();
    let kernels = list_kernels()?;
    let n = Select::with_theme(&theme)
        .items(&kernels)
        .default(kernels.len() - 1)
        .interact()?;

    install_spec_kernel(inst_path, n)?;

    Ok(())
}

fn main() -> Result<()> {
    let config = read_conf()?;
    let inst_path = Path::new(&config.esp_mountpoint).join(REL_INST_PATH);
    let matches = App::new("systemd-boot-friend-rs")
        .version(crate_version!())
        .about("Systemd-Boot Kernel Version Selector")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize systemd-boot and install the newest kernel"),
        )
        .subcommand(SubCommand::with_name("mkconf").about("Make systemd-boot config"))
        .subcommand(SubCommand::with_name("list").about("List available kernels"))
        .subcommand(
            SubCommand::with_name("install-kernel")
                .about("Install specific version of kernel")
                .arg(
                    Arg::with_name("target")
                        .help("Target kernel in the list or the number of the kernel")
                        .index(1),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", _) => init(&inst_path)?,
        ("list", _) => print_kernels()?,
        ("install-kernel", Some(args)) => {
            if let Some(n) = args.value_of("target") {
                match n.parse::<usize>() {
                    Ok(num) => install_spec_kernel(&inst_path, num - 1)?,
                    Err(_) => install_kernel(n, &inst_path)?,
                }
            } else {
                install_newest_kernel(&inst_path)?
            }
        }
        _ => ask_for_kernel(&inst_path)?,
    }

    Ok(())
}

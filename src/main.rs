use anyhow::{anyhow, Result};
use clap::{crate_version, App, SubCommand};
use serde::Deserialize;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

const CONF_PATH: &str = "/etc/systemd-boot-friend-rs.conf";

#[derive(Deserialize)]
struct Config {
    esp_mountpoint: String,
}

fn read_conf() -> Result<Config> {
    let content = fs::read(CONF_PATH)?;
    let config: Config = serde_json::from_slice(&content)?;

    Ok(config)
}

fn init(esp: &str) -> Result<()> {
    Command::new("bootctl")
        .arg("install")
        .arg("--esp=".to_owned() + esp)
        .stdout(Stdio::null())
        .spawn()?;
    fs::create_dir_all(format!("{}{}", esp, "/EFI/aosc/"))?;
    install_newest_kernel(&esp)?;

    println!("Please make sure you have written the boot entry config for systemd-boot,");
    println!("see https://systemd.io/BOOT_LOADER_SPECIFICATION/ for further information.");

    Ok(())
}

fn ls_kernels() -> Result<Vec<String>> {
    let kernels = fs::read_dir("/usr/lib/modules")?;
    let mut kernels_ls = Vec::new();
    for kernel in kernels {
        kernels_ls.push(kernel.unwrap().file_name().into_string().unwrap());
    }
    kernels_ls.sort();

    Ok(kernels_ls)
}

fn disp_kernels() -> Result<()> {
    let kernels = ls_kernels()?;
    for (i, k) in kernels.into_iter().enumerate() {
        println!("[{}] {}", i + 1, k);
    }

    Ok(())
}

fn install_kernel(kernel_name: &str, esp: &str) -> Result<()> {
    if !Path::new(esp).join("EFI/aosc/").exists() {
        println!("{}{} does not exist. Doing nothing.", esp, "/EFI/aosc/");
        println!("If you wish to use systemd-boot, run systemd-boot-friend init.");
        println!("Or, if your ESP mountpoint is not at $ESP_MOUNTPOINT, please edit /etc/systemd-boot-friend.conf.");

        return Err(anyhow!("{}{} not found", esp, "/EFI/aosc"));
    }

    let splitted_kernel_name = kernel_name.split("-").collect::<Vec<_>>();
    if splitted_kernel_name.len() != 3 {
        return Err(anyhow!("Kernel name does not meet the standard"));
    }
    let kernel_ver = splitted_kernel_name[0];
    let distro_name = splitted_kernel_name[1];
    let kernel_flavor = splitted_kernel_name[2];

    let src_vmlinuz = Path::new(&format!(
        "/boot/vmlinuz-{}-{}-{}",
        distro_name, kernel_flavor, kernel_ver
    ));
    let src_initramfs = Path::new(&format!(
        "/boot/initramfs-{}-{}-{}.img",
        kernel_ver, distro_name, kernel_flavor
    ));
    if src_vmlinuz.exists() {
        fs::copy(
            &src_vmlinuz,
            &format!(
                "{}{}vmlinuz-{}-{}",
                esp, "/EFI/aosc/", distro_name, kernel_flavor
            ),
        );
    } else {
        return Err(anyhow!("Kernel file not found"));
    }
    if src_initramfs.exists() {
        fs::copy(
            &src_initramfs,
            &format!(
                "{}{}initramfs-{}-{}.img",
                esp, "/EFI/aosc", distro_name, kernel_flavor
            ),
        );
    } else {
        return Err(anyhow!("Initramfs not found"));
    }

    Ok(())
}

fn install_newest_kernel(esp: &str) -> Result<()> {
    let kernels = ls_kernels()?;
    install_kernel(&kernels[kernels.len() - 1], esp)?;

    Ok(())
}

fn main() -> Result<()> {
    let config = read_conf()?;
    let matches = App::new("systemd-boot-friend-rs")
        .version(crate_version!())
        .about("Systemd-Boot Kernel Version Selector")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize systemd-boot and install the newest kernel"),
        )
        .subcommand(SubCommand::with_name("mkconf").about("Make systemd-boot config"))
        .subcommand(SubCommand::with_name("list").about("List available kernels"))
        .get_matches();
    match matches.subcommand_name() {
        Some("init") => init(&config.esp_mountpoint)?,
        Some("list") => disp_kernels()?,
        Some(_) => install_newest_kernel(&config.esp_mountpoint)?,
        None => install_newest_kernel(&config.esp_mountpoint)?,
    }

    Ok(())
}

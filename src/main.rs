use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use serde::Deserialize;
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
    esp_mountpoint: String,
}

/// Reads the configuration file at CONF_PATH
fn read_conf() -> Result<Config> {
    let content = fs::read(CONF_PATH)?;
    // deserialize into Config struct
    let config: Config = serde_json::from_slice(&content)?;

    Ok(config)
}

/// Initialize the default environment for friend
fn init(inst_path: &Path) -> Result<()> {
    // use bootctl to install systemd-boot
    println!("{} Initializing systemd-boot ...", OUTPUT_PREFIX);
    Command::new("bootctl")
        .arg("install")
        .arg("--esp=".to_owned() + inst_path.to_str().unwrap_or("/efi"))
        .stdout(Stdio::null())
        .spawn()?;
    // create folder structure
    println!("{} Creating folder structure for friend ...", OUTPUT_PREFIX);
    fs::create_dir_all(Path::new(inst_path).join(REL_INST_PATH))?;
    // install the newest kernel
    install_newest_kernel(&inst_path)?;

    // Currently users have to manually create the boot entry config,
    // boot entry auto generator may be implemented in the future
    println!("Please make sure you have written the boot entry config for systemd-boot,");
    println!("see https://systemd.io/BOOT_LOADER_SPECIFICATION/ for further information.");

    Ok(())
}

/// Generate a sorted vector of kernel filenames
fn list_kernels() -> Result<Vec<String>> {
    // read /usr/lib/modules to get kernel filenames
    let kernels = fs::read_dir("/usr/lib/modules")?;
    let mut kernels_ls = Vec::new();
    for kernel in kernels {
        kernels_ls.push(kernel.unwrap().file_name().into_string().unwrap());
    }
    // Sort the vector, thus the kernel filenames are
    // arranged with versions from older to newer
    kernels_ls.sort();

    Ok(kernels_ls)
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
fn install_kernel(kernel_name: &str, inst_path: &Path) -> Result<()> {
    // if the path does not exist, ask the user for initializing friend
    if !inst_path.exists() {
        println!("{} does not exist. Doing nothing.", inst_path.display());
        println!("If you wish to use systemd-boot, run systemd-boot-friend init.");
        println!("Or, if your ESP mountpoint is not at esp_mountpoint, please edit /etc/systemd-boot-friend-rs.conf.");

        return Err(anyhow!("{} not found", inst_path.display()));
    }
    // Split the kernel filename into 3 parts in order to determine
    // the version, name and the flavor of the chosen kernel
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
    // generate the path to the source files
    println!("{} Installing {} to {} ...", OUTPUT_PREFIX, kernel_name, inst_path.display());
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
    // Copy the source files to the `inst_path` using specific
    // filename format, remove the version parts of the files
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

/// Install a specific kernel to the esp using the given position in the kernel list
fn install_spec_kernel(inst_path: &Path, n: usize) -> Result<()> {
    let kernels = list_kernels()?;
    if n >= kernels.len() {
        return Err(anyhow!("Chosen kernel index out of bound"));
    }
    install_kernel(&kernels[n], inst_path)?;

    Ok(())
}

fn install_newest_kernel(inst_path: &Path) -> Result<()> {
    println!("{} Installing the newest kernel ...", OUTPUT_PREFIX);
    let kernels = list_kernels()?;
    // Install the last one in the kernel list as the list
    // has already been sorted by filename and version
    install_kernel(&kernels[kernels.len() - 1], inst_path)?;

    Ok(())
}

/// Default behavior when calling without any subcommands
fn ask_for_kernel(inst_path: &Path) -> Result<()> {
    let kernels = list_kernels()?;
    // build dialoguer Select for kernel selection
    let theme = ColorfulTheme::default();
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
    let matches = cli::build_cli().get_matches();
    // Switch table
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

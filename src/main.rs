use anyhow::{anyhow, bail, Result};
use clap::Parser;
use cli::{Opts, SubCommands};
use core::default::Default;
use dialoguer::{Confirm, MultiSelect};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use i18n::I18N_LOADER;
use kernel::{generic_kernel::GenericKernel, Kernel};

mod cli;
mod i18n;
mod kernel;
mod macros;
mod version;

const CONF_PATH: &str = "/etc/systemd-boot-friend.conf";
const REL_DEST_PATH: &str = "EFI/systemd-boot-friend/";

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
            vmlinuz: "vmlinuz-{VERSION}".to_owned(),
            initrd: "initramfs-{VERSION}.img".to_owned(),
            distro: "Linux".to_owned(),
            esp_mountpoint: PathBuf::from("/efi"),
            bootarg: String::new(),
        }
    }
}

/// Choose a kernel using dialoguer
fn choose_kernel<K: Kernel>(kernels: &[K], prompt: &str) -> Result<Vec<K>> {
    if kernels.is_empty() {
        bail!(fl!("empty_list"));
    }

    // build dialoguer MultiSelect for kernel selection
    Ok(MultiSelect::new()
        .with_prompt(prompt)
        .items(kernels)
        .interact()?
        .iter()
        .map(|n| kernels[*n].clone())
        .collect())
}

#[inline]
fn parse_num_or_filename(
    config: &Config,
    n: &str,
    kernels: &[GenericKernel],
) -> Result<GenericKernel> {
    match n.parse::<usize>() {
        Ok(num) => Ok(kernels
            .get(num - 1)
            .ok_or_else(|| anyhow!(fl!("invalid_index")))?
            .clone()),
        Err(_) => GenericKernel::parse(config, n),
    }
}

#[inline]
fn specify_or_choose(
    config: &Config,
    arg: &Option<String>,
    kernels: &[GenericKernel],
    prompt: &str,
) -> Result<Vec<GenericKernel>> {
    match arg {
        // the target can be both the number in
        // the list and the name of the kernel
        Some(n) => Ok(vec![parse_num_or_filename(config, &n, kernels)?]),
        // select the kernel to remove
        // when no target is given
        None => choose_kernel(kernels, prompt),
    }
}

/// Initialize the default environment for friend
fn init<K: Kernel>(config: &Config, installed_kernels: &[K], kernels: &[K]) -> Result<()> {
    // use bootctl to install systemd-boot
    println_with_prefix_and_fl!("init");
    print_block_with_fl!("prompt_init");

    if Confirm::new()
        .with_prompt(fl!("ask_init"))
        .default(false)
        .interact()?
    {
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
        print_block_with_fl!("prompt_update");
        Confirm::new()
            .with_prompt(fl!("ask_update"))
            .default(false)
            .interact()?
            .then(|| update(installed_kernels, kernels))
            .transpose()?;
    }

    Ok(())
}

/// Update systemd-boot kernels and entries
fn update<K: Kernel>(installed_kernels: &[K], kernels: &[K]) -> Result<()> {
    println_with_prefix_and_fl!("update");
    print_block_with_fl!("note_copy_files");

    // Remove existing kernels
    installed_kernels.iter().try_for_each(|k| k.remove())?;

    // Install all kernels
    kernels
        .iter()
        .try_for_each(|k| k.install_and_make_config(true))?;

    // Set the newest kernel as default entry
    if let Some(k) = kernels.first() {
        k.set_default()?;
    }

    Ok(())
}

#[inline]
fn install<K: Kernel>(kernel: &K, force: bool) -> Result<()> {
    print_block_with_fl!("note_copy_files");

    kernel.install_and_make_config(force)?;
    kernel.ask_set_default()?;

    Ok(())
}

#[inline]
fn print_kernels<K: Kernel>(kernels: &[K]) {
    kernels
        .iter()
        .enumerate()
        .for_each(|(i, k)| println!("\u{001b}[1m[{}]\u{001b}[0m {}", i + 1, k))
}

fn read_config() -> Result<Config> {
    match fs::read(CONF_PATH) {
        Ok(f) => {
            let mut config: Config = toml::from_slice(&f)?;

            // Migrate from old configuration
            let old_conf = "{VERSION}-{LOCALVERSION}";
            let new_conf = "{VERSION}";

            if config.vmlinuz.contains(old_conf) || config.initrd.contains(old_conf) {
                println_with_prefix_and_fl!("conf_old");
                config.vmlinuz = config.vmlinuz.replace(old_conf, new_conf);
                config.initrd = config.initrd.replace(old_conf, new_conf);
                fs::write(CONF_PATH, toml::to_string_pretty(&config)?)?;
            }

            Ok(config)
        }
        Err(_) => {
            println_with_prefix_and_fl!("conf_default", conf_path = CONF_PATH);
            fs::create_dir_all(PathBuf::from(CONF_PATH).parent().unwrap())?;
            fs::write(CONF_PATH, toml::to_string_pretty(&Config::default())?)?;
            bail!(fl!("edit_conf", conf_path = CONF_PATH))
        }
    }
}

fn main() -> Result<()> {
    // CLI
    let matches: Opts = Opts::parse();

    // Read config, create a default one if the file is missing
    let config = read_config()?;
    let installed_kernels = GenericKernel::list_installed(&config)?;
    let kernels = GenericKernel::list(&config)?;

    // Switch table
    match matches.subcommands {
        Some(s) => match s {
            SubCommands::Init => init(&config, &installed_kernels, &kernels)?,
            SubCommands::Update => update(&installed_kernels, &kernels)?,
            SubCommands::InstallKernel(args) => {
                specify_or_choose(&config, &args.target, &kernels, &fl!("select_install"))?
                    .iter()
                    .try_for_each(|k| install(k, args.force))?
            }
            SubCommands::RemoveKernel(args) => specify_or_choose(
                &config,
                &args.target,
                &installed_kernels,
                &fl!("select_remove"),
            )?
            .iter()
            .try_for_each(|k| k.remove())?,
            SubCommands::ListAvailable => print_kernels(&kernels),
            SubCommands::ListInstalled => print_kernels(&installed_kernels),
        },
        None => unreachable!(),
    }

    Ok(())
}

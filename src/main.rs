use anyhow::{anyhow, bail, Result};
use clap::Parser;
use core::default::Default;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use libsdbootconf::SystemdBootConf;
use std::{
    cell::RefCell,
    fs,
    process::{Command, Stdio},
    rc::Rc,
};

mod cli;
mod config;
mod i18n;
mod kernel;
mod kernel_manager;
mod macros;
mod version;

use cli::{Opts, SubCommands};
use config::Config;
use i18n::I18N_LOADER;
use kernel::{generic_kernel::GenericKernel, Kernel};
use kernel_manager::KernelManager;

const REL_DEST_PATH: &str = "EFI/systemd-boot-friend/";
const SRC_PATH: &str = "/boot";

/// Initialize the default environment for friend
fn init(config: &Config) -> Result<()> {
    // use bootctl to install systemd-boot
    println_with_prefix_and_fl!("init");
    print_block_with_fl!("notice_init");

    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(fl!("ask_init"))
        .default(false)
        .interact()?
    {
        return Ok(());
    }

    let child_output = Command::new("bootctl")
        .arg("install")
        .arg(
            "--esp=".to_owned()
                + config
                    .esp_mountpoint
                    .to_str()
                    .ok_or_else(|| anyhow!(fl!("invalid_esp")))?,
        )
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    if !child_output.status.success() {
        bail!(String::from_utf8(child_output.stderr)?);
    }

    let sbconf = Rc::new(RefCell::new(SystemdBootConf::new(
        config.esp_mountpoint.join("loader/"),
        libsdbootconf::Config::default(),
        Vec::new(),
    )));

    // Initialize a default config for systemd-boot
    sbconf.borrow().write_all()?;
    // Set default timeout to 5
    sbconf.borrow_mut().config.timeout = Some(5u32);
    sbconf.borrow().write_config()?;

    let installed_kernels = GenericKernel::list_installed(config, sbconf.clone())?;
    let kernels = GenericKernel::list(config, sbconf)?;

    // create folder structure
    println_with_prefix_and_fl!("create_folder");
    fs::create_dir_all(config.esp_mountpoint.join(REL_DEST_PATH))?;

    // Update systemd-boot kernels and entries
    print_block_with_fl!("prompt_update", src_path = SRC_PATH);
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(fl!("ask_update"))
        .default(false)
        .interact()?
    {
        KernelManager::new(kernels, installed_kernels).update(config)?;
    } else {
        println_with_prefix_and_fl!("skip_update");
    }

    Ok(())
}

/// Ask for the timeout of systemd-boot boot menu
fn ask_set_timeout(timeout: Option<u32>, sbconf: Rc<RefCell<SystemdBootConf>>) -> Result<()> {
    sbconf.borrow_mut().config.timeout = timeout.or_else(|| {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(fl!("input_timeout"))
            .default(5u32)
            .interact()
            .ok()
    });
    sbconf.borrow().write_config()?;

    Ok(())
}

fn main() -> Result<()> {
    // CLI
    let matches: Opts = Opts::parse();

    // Read config, create a default one if the file is missing
    let config = Config::read()?;

    // Preprocess init subcommand
    if let Some(SubCommands::Init) = &matches.subcommands {
        init(&config)?;
        return Ok(());
    }

    let sbconf = Rc::new(RefCell::new(
        SystemdBootConf::load(config.esp_mountpoint.join("loader/"))
            .map_err(|_| anyhow!(fl!("info_path_not_exist")))?,
    ));
    let installed_kernels = GenericKernel::list_installed(&config, sbconf.clone())?;
    let kernels = GenericKernel::list(&config, sbconf.clone())?;

    let kernel_manager = KernelManager::new(kernels, installed_kernels);

    // Switch table
    match matches.subcommands {
        Some(s) => match s {
            SubCommands::Init => unreachable!(), // Handled above
            SubCommands::Update => kernel_manager.update(&config)?,
            SubCommands::InstallKernel { targets, force } => kernel_manager
                .specify_or_multiselect(&config, &targets, &fl!("select_install"), sbconf)?
                .iter()
                .try_for_each(|k| KernelManager::install(k.clone(), force))?,
            SubCommands::RemoveKernel { targets } => kernel_manager
                .specify_or_multiselect(&config, &targets, &fl!("select_remove"), sbconf)?
                .iter()
                .try_for_each(|k| k.remove())?,
            SubCommands::ListAvailable => kernel_manager.list_available(),
            SubCommands::ListInstalled => kernel_manager.list_installed()?,
            SubCommands::SetDefault { target } => {
                kernel_manager
                    .specify_or_select(&config, &target, &fl!("select_default"), sbconf)?
                    .set_default()?;
            }
            SubCommands::SetTimeout { timeout } => {
                ask_set_timeout(timeout, sbconf)?;
            }
            SubCommands::Config => {
                kernel_manager
                    .select_installed_kernel(&fl!("select_default"))?
                    .set_default()?;
                ask_set_timeout(None, sbconf)?;
            }
        },
        None => unreachable!(),
    }

    Ok(())
}

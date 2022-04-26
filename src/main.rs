use anyhow::{anyhow, bail, Result};
use clap::Parser;
use console::style;
use core::default::Default;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
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
mod macros;
mod version;

use cli::{Opts, SubCommands};
use config::Config;
use i18n::I18N_LOADER;
use kernel::{generic_kernel::GenericKernel, Kernel};

const REL_DEST_PATH: &str = "EFI/systemd-boot-friend/";
const SRC_PATH: &str = "/boot";

/// Choose kernels using dialoguer
#[inline]
fn multiselect_kernel<K: Kernel>(kernels: &[Rc<K>], prompt: &str) -> Result<Vec<Rc<K>>> {
    if kernels.is_empty() {
        bail!(fl!("empty_list"));
    }

    // build dialoguer MultiSelect for kernel selection
    Ok(MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(kernels)
        .interact()?
        .iter()
        .map(|n| kernels[*n].clone())
        .collect())
}

/// Choose a kernel using dialoguer
#[inline]
fn select_kernel<K: Kernel>(kernels: &[Rc<K>], prompt: &str) -> Result<Rc<K>> {
    if kernels.is_empty() {
        bail!(fl!("empty_list"));
    }

    // build dialoguer MultiSelect for kernel selection
    Ok(kernels[Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(kernels)
        .interact()?]
    .clone())
}

fn specify_or_multiselect(
    config: Rc<Config>,
    arg: &[String],
    kernels: &[Rc<GenericKernel>],
    prompt: &str,
    sbconf: Rc<RefCell<SystemdBootConf>>,
) -> Result<Vec<Rc<GenericKernel>>> {
    if arg.is_empty() {
        // select the kernels when no target is given
        multiselect_kernel(kernels, prompt)
    } else {
        let mut kernels = Vec::new();

        for target in arg {
            kernels.push(Rc::new(GenericKernel::parse(
                config.clone(),
                target,
                sbconf.clone(),
            )?));
        }

        Ok(kernels)
    }
}

#[inline]
fn specify_or_select(
    config: Rc<Config>,
    arg: &Option<String>,
    kernels: &[Rc<GenericKernel>],
    prompt: &str,
    sbconf: Rc<RefCell<SystemdBootConf>>,
) -> Result<Rc<GenericKernel>> {
    match arg {
        // parse the kernel name when a target is given
        Some(n) => Ok(Rc::new(GenericKernel::parse(config, n, sbconf)?)),
        // select the kernel when no target is given
        None => select_kernel(kernels, prompt),
    }
}

/// Initialize the default environment for friend
fn init(config: Rc<Config>) -> Result<()> {
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
        &config.esp_mountpoint.join("loader/"),
        libsdbootconf::Config::default(),
        Vec::new(),
    )));

    // Initialize a default config for systemd-boot
    sbconf.borrow().write_all()?;
    // Set default timeout to 5
    sbconf.borrow_mut().config.timeout = Some(5u32);
    sbconf.borrow().write_config()?;

    let installed_kernels = GenericKernel::list_installed(config.clone(), sbconf.clone())?;
    let kernels = GenericKernel::list(config.clone(), sbconf)?;

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
        update(&kernels, &installed_kernels)?;
    } else {
        println_with_prefix_and_fl!("skip_update");
    }

    Ok(())
}

/// Update systemd-boot kernels and entries
fn update<K: Kernel>(kernels: &[Rc<K>], installed_kernels: &[Rc<K>]) -> Result<()> {
    println_with_prefix_and_fl!("update");
    print_block_with_fl!("note_copy_files");

    // Remove obsoleted kernels
    installed_kernels.iter().try_for_each(|k| {
        if !kernels.contains(k) {
            k.remove()
        } else {
            Ok(())
        }
    })?;

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
fn install<K: Kernel>(kernel: Rc<K>, force: bool) -> Result<()> {
    print_block_with_fl!("note_copy_files");

    kernel.install_and_make_config(force)?;
    kernel.ask_set_default()?;

    Ok(())
}

/// Print all the available kernels
fn list_available<K: Kernel>(kernels: &[Rc<K>], installed_kernels: &[Rc<K>]) {
    if !kernels.is_empty() {
        for k in kernels.iter() {
            if installed_kernels.contains(k) {
                print!("{} ", style("[*]").green());
            } else {
                print!("[ ] ");
            }
            println!("{}", k);
        }
        println!();
        println_with_fl!("note_list_available");
    }
}

/// Print all the installed kernels
fn list_installed<K: Kernel>(installed_kernels: &[Rc<K>]) {
    if !installed_kernels.is_empty() {
        for k in installed_kernels.iter() {
            if k.is_default() {
                print!("{} ", style("[*]").green());
            } else {
                print!("[ ] ");
            }
            println!("{}", k);
        }
        println!();
        println_with_fl!("note_list_installed");
    }
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
    let config = Rc::new(Config::read()?);

    // Preprocess init subcommand
    if let Some(SubCommands::Init) = &matches.subcommands {
        init(config)?;
        return Ok(());
    }

    let sbconf = Rc::new(RefCell::new(SystemdBootConf::load(
        &config.esp_mountpoint.join("loader/"),
    )?));
    let installed_kernels = GenericKernel::list_installed(config.clone(), sbconf.clone())?;
    let kernels = GenericKernel::list(config.clone(), sbconf.clone())?;

    // Switch table
    match matches.subcommands {
        Some(s) => match s {
            SubCommands::Init => unreachable!(),
            SubCommands::Update => update(&kernels, &installed_kernels)?,
            SubCommands::InstallKernel(args) => specify_or_multiselect(
                config,
                &args.targets,
                &kernels,
                &fl!("select_install"),
                sbconf,
            )?
            .iter()
            .try_for_each(|k| install(k.clone(), args.force))?,
            SubCommands::RemoveKernel(args) => specify_or_multiselect(
                config,
                &args.targets,
                &installed_kernels,
                &fl!("select_remove"),
                sbconf,
            )?
            .iter()
            .try_for_each(|k| k.remove())?,
            SubCommands::ListAvailable => list_available(&kernels, &installed_kernels),
            SubCommands::ListInstalled => list_installed(&installed_kernels),
            SubCommands::SetDefault(args) => {
                specify_or_select(
                    config,
                    &args.target,
                    &installed_kernels,
                    &fl!("select_default"),
                    sbconf,
                )?
                .set_default()?;
            }
            SubCommands::SetTimeout(args) => {
                ask_set_timeout(args.timeout, sbconf)?;
            }
            SubCommands::Config => {
                select_kernel(&installed_kernels, &fl!("select_default"))?.set_default()?;
                ask_set_timeout(None, sbconf)?;
            }
        },
        None => unreachable!(),
    }

    Ok(())
}

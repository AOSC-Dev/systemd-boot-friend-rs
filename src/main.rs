use anyhow::{anyhow, bail, Result};
use clap::Parser;
use console::{style, Style};
use core::default::Default;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use std::{
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

/// Modified version of ColorfulTheme for Dialoguer
#[allow(clippy::field_reassign_with_default)]
fn colorful_theme_modded() -> ColorfulTheme {
    let mut theme = ColorfulTheme::default();
    theme.prompt_suffix = style("›".to_string()).for_stderr().white();
    theme.success_suffix = style("·".to_string()).for_stderr().white();
    theme.hint_style = Style::new().for_stderr().white();
    theme.unchecked_item_prefix = style("✔".to_string()).for_stderr().black().bright();

    theme
}

/// Choose a kernel using dialoguer
fn choose_kernel<K: Kernel>(kernels: &[Rc<K>], prompt: &str) -> Result<Vec<Rc<K>>> {
    if kernels.is_empty() {
        bail!(fl!("empty_list"));
    }

    // build dialoguer MultiSelect for kernel selection
    Ok(MultiSelect::with_theme(&colorful_theme_modded())
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
    kernels: &[Rc<GenericKernel>],
) -> Result<Rc<GenericKernel>> {
    match n.parse::<usize>() {
        Ok(num) => Ok(kernels
            .get(num - 1)
            .ok_or_else(|| anyhow!(fl!("invalid_index")))?
            .clone()),
        Err(_) => Ok(Rc::new(GenericKernel::parse(config, n)?)),
    }
}

#[inline]
fn specify_or_choose(
    config: &Config,
    arg: &Option<String>,
    kernels: &[Rc<GenericKernel>],
    prompt: &str,
) -> Result<Vec<Rc<GenericKernel>>> {
    match arg {
        // the target can be both the number in
        // the list and the name of the kernel
        Some(n) => Ok(vec![parse_num_or_filename(config, n, kernels)?]),
        // select the kernel to remove
        // when no target is given
        None => choose_kernel(kernels, prompt),
    }
}

/// Initialize the default environment for friend
fn init<K: Kernel>(config: &Config, installed_kernels: &[Rc<K>], kernels: &[Rc<K>]) -> Result<()> {
    // use bootctl to install systemd-boot
    println_with_prefix_and_fl!("init");
    print_block_with_fl!("prompt_init");

    if !Confirm::with_theme(&colorful_theme_modded())
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

    // create folder structure
    println_with_prefix_and_fl!("create_folder");
    fs::create_dir_all(config.esp_mountpoint.join(REL_DEST_PATH))?;

    // Update systemd-boot kernels and entries
    print_block_with_fl!("prompt_update", src_path = SRC_PATH);
    Confirm::with_theme(&colorful_theme_modded())
        .with_prompt(fl!("ask_update"))
        .default(false)
        .interact()?
        .then(|| update(installed_kernels, kernels))
        .transpose()?;

    Ok(())
}

/// Update systemd-boot kernels and entries
fn update<K: Kernel>(installed_kernels: &[Rc<K>], kernels: &[Rc<K>]) -> Result<()> {
    println_with_prefix_and_fl!("update");
    print_block_with_fl!("note_copy_files");

    // Install all kernels
    kernels
        .iter()
        .try_for_each(|k| k.install_and_make_config(true))?;

    // Remove obsoleted kernels
    installed_kernels.iter().try_for_each(|k| {
        if !kernels.contains(k) {
            k.remove()
        } else {
            Ok(())
        }
    })?;

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

#[inline]
fn print_kernels<K: Kernel>(kernels: &[Rc<K>]) {
    kernels
        .iter()
        .enumerate()
        .for_each(|(i, k)| println!("\u{001b}[1m[{}]\u{001b}[0m {}", i + 1, k))
}

fn main() -> Result<()> {
    // CLI
    let matches: Opts = Opts::parse();

    // Read config, create a default one if the file is missing
    let config = Config::read()?;
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
                    .try_for_each(|k| install(k.clone(), args.force))?
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

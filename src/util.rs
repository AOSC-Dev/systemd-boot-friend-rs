use crate::{config::Config, fl, kernel::Kernel};
use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use libsdbootconf::SystemdBootConf;
use std::{cell::RefCell, rc::Rc};

pub fn multiselect_kernel<K: Kernel>(
    kernels: &[K],
    installed_kernels: &[K],
    prompt: &str,
) -> Result<Vec<K>> {
    if kernels.is_empty() {
        bail!(fl!("empty_list"));
    }

    // build dialoguer MultiSelect for kernel selection
    Ok(MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(kernels)
        .defaults(
            &kernels
                .iter()
                .map(|k| installed_kernels.contains(k))
                .collect::<Vec<bool>>(),
        )
        .interact()?
        .iter()
        .map(|n| kernels[*n].clone())
        .collect())
}

/// Choose a kernel using dialoguer
pub fn select_kernel<K: Kernel>(kernels: &[K], prompt: &str) -> Result<K> {
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

pub fn specify_or_multiselect<K: Kernel>(
    kernels: &[K],
    config: &Config,
    arg: &[String],
    prompt: &str,
    sbconf: Rc<RefCell<SystemdBootConf>>,
) -> Result<Vec<K>> {
    if arg.is_empty() {
        // select the kernels when no target is given
        multiselect_kernel(kernels, &[], prompt)
    } else {
        let mut kernels = Vec::new();

        for target in arg {
            kernels.push(K::parse(config, target, sbconf.clone())?);
        }

        Ok(kernels)
    }
}

pub fn specify_or_select<K: Kernel>(
    kernels: &[K],
    config: &Config,
    arg: &Option<String>,
    prompt: &str,
    sbconf: Rc<RefCell<SystemdBootConf>>,
) -> Result<K> {
    match arg {
        // parse the kernel name when a target is given
        Some(n) => Ok(K::parse(config, n, sbconf)?),
        // select the kernel when no target is given
        None => select_kernel(kernels, prompt),
    }
}

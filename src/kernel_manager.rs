use anyhow::{bail, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use libsdbootconf::SystemdBootConf;
use std::{cell::RefCell, rc::Rc};

use crate::{
    fl, kernel::Kernel, print_block_with_fl, println_with_fl, println_with_prefix_and_fl, Config,
    println_with_prefix
};

/// Manage kernels
pub struct KernelManager<K: Kernel> {
    kernels: Vec<Rc<K>>,
    installed_kernels: Vec<Rc<K>>,
}

impl<K: Kernel> KernelManager<K> {
    /// Create a new Kernel Manager
    pub fn new(kernels: Vec<Rc<K>>, installed_kernels: Vec<Rc<K>>) -> Self {
        Self {
            kernels,
            installed_kernels,
        }
    }

    /// Choose kernels using dialoguer
    #[inline]
    pub fn multiselect_kernel(&self, prompt: &str) -> Result<Vec<Rc<K>>> {
        if self.kernels.is_empty() {
            bail!(fl!("empty_list"));
        }

        // build dialoguer MultiSelect for kernel selection
        Ok(MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(&self.kernels)
            .interact()?
            .iter()
            .map(|n| self.kernels[*n].clone())
            .collect())
    }

    /// Choose a kernel using dialoguer
    #[inline]
    fn select_kernel(&self, kernels: &[Rc<K>], prompt: &str) -> Result<Rc<K>> {
        if kernels.is_empty() {
            bail!(fl!("empty_list"));
        }

        // build dialoguer MultiSelect for kernel selection
        Ok(self.kernels[Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(kernels)
            .interact()?]
        .clone())
    }

    /// Choose a kernel from available kernels
    #[inline]
    pub fn select_available_kernel(&self, prompt: &str) -> Result<Rc<K>> {
        self.select_kernel(&self.kernels, prompt)
    }

    /// Choose a kernel from installed kernels
    #[inline]
    pub fn select_installed_kernel(&self, prompt: &str) -> Result<Rc<K>> {
        self.select_kernel(&self.installed_kernels, prompt)
    }

    pub fn specify_or_multiselect(
        &self,
        config: &Config,
        arg: &[String],
        prompt: &str,
        sbconf: Rc<RefCell<SystemdBootConf>>,
    ) -> Result<Vec<Rc<K>>> {
        if arg.is_empty() {
            // select the kernels when no target is given
            self.multiselect_kernel(prompt)
        } else {
            let mut kernels = Vec::new();

            for target in arg {
                kernels.push(Rc::new(K::parse(config, target, sbconf.clone())?));
            }

            Ok(kernels)
        }
    }

    #[inline]
    pub fn specify_or_select(
        &self,
        config: &Config,
        arg: &Option<String>,
        prompt: &str,
        sbconf: Rc<RefCell<SystemdBootConf>>,
    ) -> Result<Rc<K>> {
        match arg {
            // parse the kernel name when a target is given
            Some(n) => Ok(Rc::new(K::parse(config, n, sbconf)?)),
            // select the kernel when no target is given
            None => self.select_available_kernel(prompt),
        }
    }

    /// Update systemd-boot kernels and entries
    pub fn update(&self) -> Result<()> {
        println_with_prefix_and_fl!("update");
        print_block_with_fl!("note_copy_files");

        // Remove obsoleted kernels
        self.installed_kernels.iter().try_for_each(|k| {
            if !self.kernels.contains(k) {
                k.remove()
            } else {
                Ok(())
            }
        })?;

        // Install all kernels
        self.kernels
            .iter()
            .try_for_each(|k| k.install_and_make_config(true))?;

        // Set the newest kernel as default entry
        if let Some(k) = self.kernels.first() {
            k.set_default()?;
        }

        Ok(())
    }

    #[inline]
    pub fn install(kernel: Rc<K>, force: bool) -> Result<()> {
        print_block_with_fl!("note_copy_files");

        kernel.install_and_make_config(force)?;
        kernel.ask_set_default()?;

        Ok(())
    }

    /// Print all the available kernels
    pub fn list_available(&self) {
        if !self.kernels.is_empty() {
            for k in self.kernels.iter() {
                if self.installed_kernels.contains(k) {
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
    pub fn list_installed(&self) -> Result<()> {
        if !self.installed_kernels.is_empty() {
            for k in self.installed_kernels.iter() {
                if k.is_default()? {
                    print!("{} ", style("[*]").green());
                } else {
                    print!("[ ] ");
                }
                println!("{}", k);
            }
            println!();
            println_with_fl!("note_list_installed");
        }

        Ok(())
    }
}

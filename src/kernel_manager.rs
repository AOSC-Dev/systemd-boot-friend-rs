use anyhow::Result;
use console::style;

use crate::{
    fl, kernel::Kernel, print_block_with_fl, println_with_fl, println_with_prefix,
    println_with_prefix_and_fl, Config,
};

/// Manage kernels
pub struct KernelManager<'a, K: Kernel> {
    kernels: &'a [K],
    installed_kernels: &'a [K],
}

impl<'a, K: Kernel> KernelManager<'a, K> {
    /// Create a new Kernel Manager
    pub fn new(kernels: &'a [K], installed_kernels: &'a [K]) -> Self {
        Self {
            kernels,
            installed_kernels,
        }
    }

    /// Update systemd-boot kernels and entries
    pub fn update(&self, config: &Config) -> Result<()> {
        println_with_prefix_and_fl!("update");
        print_block_with_fl!("note_copy_files");

        let keep = config
            .keep
            .unwrap_or(self.kernels.len())
            .min(self.kernels.len());

        let to_be_installed = &self.kernels[..keep];

        // Remove obsoleted kernels
        self.installed_kernels.iter().try_for_each(|k| {
            if !to_be_installed.contains(k) {
                k.remove()
            } else {
                Ok(())
            }
        })?;

        // Install all kernels
        self.kernels
            .iter()
            .take(keep)
            .try_for_each(|k| k.install_and_make_config(true))?;

        // Set the newest kernel as default entry
        if let Some(k) = self.kernels.first() {
            k.set_default()?;
        }

        Ok(())
    }

    #[inline]
    pub fn install(kernel: &K, force: bool) -> Result<()> {
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

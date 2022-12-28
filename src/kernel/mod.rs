use anyhow::Result;
use libsdbootconf::SystemdBootConf;
use same_file::is_same_file;
use std::{cell::RefCell, fmt::Display, fs, path::Path, rc::Rc};

use crate::config::Config;

const REL_ENTRY_PATH: &str = "loader/entries/";

pub trait Kernel: Display + Clone + PartialEq {
    fn parse(
        config: &Config,
        kernel_name: &str,
        sbconf: Rc<RefCell<SystemdBootConf>>,
    ) -> Result<Self>;
    fn install(&self) -> Result<()>;
    fn remove(&self) -> Result<()>;
    fn make_config(&self, force_write: bool) -> Result<()>;
    fn set_default(&self) -> Result<()>;
    fn remove_default(&self) -> Result<()>;
    fn ask_set_default(&self) -> Result<()>;
    fn is_default(&self) -> Result<bool>;
    fn install_and_make_config(&self, force_write: bool) -> Result<()>;
}

pub fn file_copy<P, Q>(src: P, dest: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    // Only copy if the dest file is missing / different
    if !dest.as_ref().exists() || !is_same_file(&src, &dest)? {
        fs::copy(&src, &dest)?;
    }

    Ok(())
}

pub mod generic_kernel;

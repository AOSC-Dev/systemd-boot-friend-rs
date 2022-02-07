use anyhow::{bail, Result};
use std::{fmt::Display, fs, path::Path};

use crate::fl;

pub trait Kernel: Display + Clone {
    fn install(&self) -> Result<()>;
    fn remove(&self) -> Result<()>;
    fn make_config(&self, force_write: bool) -> Result<()>;
    fn set_default(&self) -> Result<()>;
    fn remove_default(&self) -> Result<()>;
    fn ask_set_default(&self) -> Result<()>;
    fn install_and_make_config(&self, force_write: bool) -> Result<()>;
}

// Make sure the copy is complete, otherwise possible ENOSPC (No space left on device)
pub fn safe_copy<P: AsRef<Path>>(src: P, dest: P) -> Result<()> {
    if fs::metadata(&src)?.len() != fs::copy(&src, &dest)? {
        // Remove incomplete copy
        fs::remove_file(&dest)?;
        bail!(fl!("no_space"));
    }

    Ok(())
}

pub mod generic_kernel;

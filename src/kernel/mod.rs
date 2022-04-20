use anyhow::{bail, Result};
use sha2::{Sha256, Digest};
use std::{fmt::Display, fs, io, path::Path};

use crate::fl;

const REL_ENTRY_PATH: &str = "loader/entries/";

pub trait Kernel: Display + Clone + PartialEq {
    fn install(&self) -> Result<()>;
    fn remove(&self) -> Result<()>;
    fn make_config(&self, force_write: bool) -> Result<()>;
    fn set_default(&self) -> Result<()>;
    fn remove_default(&self) -> Result<()>;
    fn ask_set_default(&self) -> Result<()>;
    fn install_and_make_config(&self, force_write: bool) -> Result<()>;
}

// Check if two files are identical
fn same_files<P: AsRef<Path>>(file1: P, file2: P) -> Result<bool> {
    let mut hasher1 = Sha256::new();
    io::copy(&mut fs::File::open(&file1)?, &mut hasher1)?;
    let hash1 = hasher1.finalize();

    let mut hasher2 = Sha256::new();
    io::copy(&mut fs::File::open(&file2)?, &mut hasher2)?;
    let hash2 = hasher2.finalize();

    Ok(hash1 == hash2)
}

// Make sure the copy is complete, otherwise possible ENOSPC (No space left on device)
pub fn safe_copy<P: AsRef<Path>>(src: P, dest: P) -> Result<()> {
    if dest.as_ref().exists()
        && !same_files(&src, &dest)?
        && fs::metadata(&src)?.len() != fs::copy(&src, &dest)?
    {
        // Remove incomplete copy
        fs::remove_file(&dest)?;
        bail!(fl!("no_space"));
    }

    Ok(())
}

pub mod generic_kernel;

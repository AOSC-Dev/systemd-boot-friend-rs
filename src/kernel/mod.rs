use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{fmt::Display, fs, io, path::Path};

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

pub fn file_copy<P: AsRef<Path>>(src: P, dest: P) -> Result<()> {
    // Only copy if the dest file is missing / different
    if !dest.as_ref().exists() || !same_files(&src, &dest)? {
        fs::copy(&src, &dest)?;
    }

    Ok(())
}

pub mod generic_kernel;

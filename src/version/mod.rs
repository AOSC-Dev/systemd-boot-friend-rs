use anyhow::Result;
use std::fmt::Display;

pub trait Version: Display + Sized {
    fn parse(input: &str) -> Result<Self>;
}

#[cfg(feature = "generic")]
pub mod generic_version;

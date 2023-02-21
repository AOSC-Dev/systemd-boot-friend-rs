use anyhow::Result;
use std::fmt::Display;

pub trait Version: Display + Sized {
    fn parse(input: &str) -> Result<Self>;
}

pub mod generic_version;

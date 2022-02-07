use anyhow::Result;

pub trait Version<T> {
    fn parse(input: &str) -> Result<T>;
}

pub mod generic_version;

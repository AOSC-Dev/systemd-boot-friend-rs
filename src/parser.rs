use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use crate::kernel::Version;

#[derive(Parser)]
#[grammar = "kernel.pest"]
struct VersionParser;

pub fn parse_version(raw_version: &str) -> Result<(Version, String)> {
    let pairs = VersionParser::parse(Rule::version, raw_version)?;
    let mut version = Version::default();
    let mut localver = String::new();
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::major => version.major = inner_pair.as_str().parse::<u64>()?,
                Rule::minor => version.minor = inner_pair.as_str().parse::<u64>()?,
                Rule::patch => version.patch = inner_pair.as_str().parse::<u64>()?,
                Rule::rc => version.rc = Some(inner_pair.as_str().to_owned()),
                Rule::localver => localver = inner_pair.as_str().to_owned(),
                _ => unreachable!(),
            }
        }
    }

    Ok((version, localver))
}

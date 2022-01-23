use anyhow::{anyhow, Result};
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::digit1,
    combinator::{map, map_res, opt},
    sequence::{pair, preceded, tuple},
    IResult,
};
use std::fmt;

use crate::fl;

#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub rc: Option<String>,
    pub rel: Option<u64>,
    pub localversion: String,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}{}{}{}",
            self.major,
            self.minor,
            self.patch,
            self.rc
                .as_ref()
                .map_or_else(|| "".to_owned(), |s| format!("-{}", s)),
            self.rel
                .as_ref()
                .map_or_else(|| "".to_owned(), |s| format!("-{}", s)),
            self.localversion
        )
    }
}

fn version_digit(input: &str) -> IResult<&str, u64> {
    map_res(digit1, |x: &str| x.parse())(input)
}

fn digit_after_dot(input: &str) -> IResult<&str, u64> {
    preceded(tag("."), version_digit)(input)
}

fn rc(input: &str) -> IResult<&str, String> {
    map(preceded(tag("-"), pair(tag("rc"), digit1)), |(x, y)| {
        format!("{}{}", x, y)
    })(input)
}

fn rel(input: &str) -> IResult<&str, u64> {
    map_res(preceded(tag("-"), take_until("-")), |x: &str| x.parse())(input)
}

fn version(input: &str) -> IResult<&str, Version> {
    tuple((
        version_digit,        // Major
        digit_after_dot,      // Minor
        opt(digit_after_dot), // Optional Patch
        opt(rc),              // Optional RC
        opt(rel),             // Optional Rel
    ))(input)
    .map(|(next, res)| {
        let (major, minor, patch, rc, rel) = res;
        let version = Version {
            major,
            minor,
            patch: patch.unwrap_or_default(),
            rc,
            rel,
            localversion: next.to_owned(),
        };
        (next, version)
    })
}

impl Version {
    pub fn parse(input: &str) -> Result<Version> {
        version(input)
            .map(|(_, version)| version)
            .map_err(|_| anyhow!(fl!("invalid_kernel_filename")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_aosc_version() {
        assert_eq!(
            version("5.12.0-rc3-aosc-main"),
            Ok((
                "-aosc-main",
                Version {
                    major: 5,
                    minor: 12,
                    patch: 0,
                    rc: Some("rc3".to_owned()),
                    rel: None,
                    localversion: "-aosc-main".to_owned(),
                }
            ))
        );
        assert_eq!(
            version("5.12-aosc-main"),
            Ok((
                "-aosc-main",
                Version {
                    major: 5,
                    minor: 12,
                    patch: 0,
                    rc: None,
                    rel: None,
                    localversion: "-aosc-main".to_owned(),
                }
            ))
        );
    }

    #[test]
    fn test_fedora_version() {
        assert_eq!(
            version("5.15.12-100.fc34.x86_64"),
            Ok((
                "-100.fc34.x86_64",
                Version {
                    major: 5,
                    minor: 15,
                    patch: 12,
                    rc: None,
                    rel: None,
                    localversion: "-100.fc34.x86_64".to_owned(),
                }
            ))
        );
    }

    #[test]
    fn test_debian_version() {
        assert_eq!(
            version("5.10.0-11-amd64"),
            Ok((
                "-amd64",
                Version {
                    major: 5,
                    minor: 10,
                    patch: 0,
                    rc: None,
                    rel: Some(11),
                    localversion: "-amd64".to_owned(),
                }
            ))
        );
    }
}

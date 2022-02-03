use anyhow::{anyhow, Result};
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::digit1,
    combinator::{map_res, opt},
    sequence::{preceded, tuple},
    IResult,
};
use std::fmt;

use crate::fl;

#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub rc: Option<u64>,
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

fn rc(input: &str) -> IResult<&str, u64> {
    preceded(tag("-rc"), version_digit)(input)
}

fn rel(input: &str) -> IResult<&str, u64> {
    map_res(preceded(tag("-"), take_until("-")), |x: &str| x.parse())(input)
}

impl Version {
    pub fn parse(input: &str) -> Result<Version> {
        tuple((
            version_digit,        // Major
            digit_after_dot,      // Minor
            opt(digit_after_dot), // Optional Patch
            opt(rc),              // Optional RC
            opt(rel),             // Optional Rel
        ))(input)
        .map_or_else(
            |_| Err(anyhow!(fl!("invalid_kernel_filename"))),
            |(next, res)| {
                let (major, minor, patch, rc, rel) = res;
                let version = Version {
                    major,
                    minor,
                    patch: patch.unwrap_or_default(),
                    rc,
                    rel,
                    localversion: next.into(),
                };

                Ok(version)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_aosc_version() {
        assert_eq!(
            Version::parse("5.12.0-rc3-aosc-main").unwrap(),
            Version {
                major: 5,
                minor: 12,
                patch: 0,
                rc: Some(3),
                rel: None,
                localversion: "-aosc-main".to_owned(),
            }
        );
        assert_eq!(
            Version::parse("5.12-aosc-main").unwrap(),
            Version {
                major: 5,
                minor: 12,
                patch: 0,
                rc: None,
                rel: None,
                localversion: "-aosc-main".to_owned(),
            }
        );
    }

    #[test]
    fn test_fedora_version() {
        assert_eq!(
            Version::parse("5.15.12-100.fc34.x86_64").unwrap(),
            Version {
                major: 5,
                minor: 15,
                patch: 12,
                rc: None,
                rel: None,
                localversion: "-100.fc34.x86_64".to_owned(),
            }
        );
    }

    #[test]
    fn test_debian_version() {
        assert_eq!(
            Version::parse("5.10.0-11-amd64").unwrap(),
            Version {
                major: 5,
                minor: 10,
                patch: 0,
                rc: None,
                rel: Some(11),
                localversion: "-amd64".to_owned(),
            }
        );
    }
}

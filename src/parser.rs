// Nom Parser
use anyhow::{anyhow, Result};
use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::opt,
    sequence::{pair, preceded, tuple},
    IResult,
};

use crate::{fl, kernel::Version};

fn version_digit(input: &str) -> IResult<&str, u64> {
    digit1(input).map(|(next, x)| (next, x.parse::<u64>().unwrap()))
}

fn digit_after_dot(input: &str) -> IResult<&str, u64> {
    preceded(tag("."), version_digit)(input)
}

fn rc(input: &str) -> IResult<&str, String> {
    preceded(tag("-"), pair(tag("rc"), digit1))(input)
        .map(|(next, (x, y))| (next, format!("{}{}", x, y)))
}

fn version(input: &str) -> IResult<&str, Version> {
    tuple((
        version_digit,        // Major
        digit_after_dot,      // Minor
        opt(digit_after_dot), // Optional Patch
        opt(rc),              // Optional RC
    ))(input)
    .map(|(next, res)| {
        let (major, minor, patch, rc) = res;
        let version = Version {
            major,
            minor,
            patch: patch.unwrap_or(0),
            rc,
            localversion: next.to_owned(),
        };
        (next, version)
    })
}

pub fn parse_version(input: &str) -> Result<Version> {
    version(input)
        .map(|(_, version)| version)
        .map_err(|_| anyhow!(fl!("invalid_kernel_filename")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_version() {
        assert_eq!(
            version("5.12.0-rc3-aosc-main"),
            Ok((
                "-aosc-main",
                Version {
                    major: 5,
                    minor: 12,
                    patch: 0,
                    rc: Some("rc3".to_owned()),
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
                    localversion: "-aosc-main".to_owned(),
                }
            ))
        );
        assert_eq!(
            version("5.15.12-100.fc34.x86_64"),
            Ok((
                "-100.fc34.x86_64",
                Version {
                    major: 5,
                    minor: 15,
                    patch: 12,
                    rc: None,
                    localversion: "-100.fc34.x86_64".to_owned(),
                }
            ))
        );
    }
}

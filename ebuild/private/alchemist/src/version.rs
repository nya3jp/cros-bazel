// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, Error, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    str::FromStr,
};

pub const VERSION_RE_RAW: &'static str =
    r"[0-9]+(?:\.[0-9]+)*[a-z]?(?:_(?:alpha|beta|pre|rc|p)[0-9]*)*(?:-r[0-9]+)?";
static VERSION_SUFFIX_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!("-{}$", VERSION_RE_RAW)).unwrap());

/// Represents a version of Portage packages.
///
/// See PMS for the specification.
/// https://projects.gentoo.org/pms/8/pms.html#x1-250003.2
#[derive(Clone, Debug, Hash)]
pub struct Version {
    main: Vec<String>,
    letter: String,
    suffixes: Vec<VersionSuffix>,
    revision: String,
}

impl Version {
    /// Parses `text` into [`Version`].
    ///
    /// [`Version`] also implements the [`FromStr`] trait, which allows you to
    /// use `str::parse` to convert [`str`] into [`Version`].
    pub fn try_new(text: &str) -> Result<Self> {
        let (_, ver) = parser::parse_version(text).map_err(|e| e.to_owned())?;
        Ok(ver)
    }

    /// Extracts a version suffix from `input` and returns a pair of the
    /// prefix and [`Version`].
    ///
    /// A hyphen should separate the prefix and the version suffix.
    ///
    /// # Example
    ///
    /// ```
    /// # use alchemist::version::Version;
    /// assert_eq!(("sys-apps/attr", Version::try_new("2.5.1")?), Version::from_str_suffix("sys-apps/attr-2.5.1")?);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_str_suffix(input: &str) -> Result<(&str, Self)> {
        let caps = VERSION_SUFFIX_RE
            .captures(input)
            .ok_or_else(|| anyhow!("invalid version number"))?;
        let ver = Self::try_new(&caps[0][1..])?;
        Ok((&input[..caps.get(0).unwrap().start()], ver))
    }

    /// Returns the main part of the version. This is referred to as "numeric
    /// components" in PMS.
    ///
    /// # Example
    ///
    /// ```
    /// # use alchemist::version::Version;
    /// assert_eq!(&vec!["1".to_string(), "2".to_string(), "3".to_string()], Version::try_new("1.2.3g_beta7_p4-r8")?.main());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn main(&self) -> &Vec<String> {
        &self.main
    }

    /// Returns the letter part of the version.
    ///
    /// # Example
    ///
    /// ```
    /// # use alchemist::version::Version;
    /// assert_eq!("", Version::try_new("1.2.3_beta7_p4-r8")?.letter());
    /// assert_eq!("g", Version::try_new("1.2.3g_beta7_p4-r8")?.letter());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn letter(&self) -> &str {
        &self.letter
    }

    /// Returns the suffixes part of the version.
    ///
    /// # Example
    ///
    /// ```
    /// # use alchemist::version::{Version, VersionSuffix, VersionSuffixLabel};
    /// let version = Version::try_new("1.2.3g_beta7_p4-r8")?;
    /// let suffixes = version.suffixes();
    /// assert_eq!(2, suffixes.len());
    /// assert_eq!(VersionSuffixLabel::Beta, suffixes[0].label());
    /// assert_eq!("7", suffixes[0].number());
    /// assert_eq!(VersionSuffixLabel::P, suffixes[1].label());
    /// assert_eq!("4", suffixes[1].number());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn suffixes(&self) -> &Vec<VersionSuffix> {
        &self.suffixes
    }

    /// Returns the revision part of the version.
    ///
    /// # Example
    ///
    /// ```
    /// # use alchemist::version::Version;
    /// assert_eq!("", Version::try_new("1.2.3g_beta7_p4")?.revision());
    /// assert_eq!("8", Version::try_new("1.2.3g_beta7_p4-r8")?.revision());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Returns a copy of [`Version`] without the revision part.
    ///
    /// # Example
    ///
    /// ```
    /// # use alchemist::version::Version;
    /// assert_eq!("1.2.3g_beta7_p4", Version::try_new("1.2.3g_beta7_p4-r8")?.without_revision().to_string());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn without_revision(&self) -> Self {
        Self {
            main: self.main.clone(),
            letter: self.letter.clone(),
            suffixes: self.suffixes.clone(),
            revision: String::new(),
        }
    }

    /// Checks if the [`Version`] has `prefix` as a prefix.
    ///
    /// # Example
    ///
    /// ```
    /// # use alchemist::version::Version;
    /// assert_eq!(true, Version::try_new("1.2.3g_beta7_p4-r8")?.starts_with(&Version::try_new("1.2.3g_beta7")?));
    /// assert_eq!(true, Version::try_new("1.2.3g_beta7_p4-r8")?.starts_with(&Version::try_new("1.2")?));
    /// assert_eq!(false, Version::try_new("1.2.3g_beta7_p4-r8")?.starts_with(&Version::try_new("1.2.4")?));
    /// assert_eq!(false, Version::try_new("1.2.3g_beta7_p4-r8")?.starts_with(&Version::try_new("1.2.3g_p4-r8")?));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn starts_with(&self, prefix: &Version) -> bool {
        let trimed_version = (|| {
            let mut copy = self.clone();

            if prefix.revision != "" {
                return copy;
            }
            copy.revision.clear();

            if copy.suffixes.len() > prefix.suffixes.len() {
                copy.suffixes.truncate(prefix.suffixes.len());
            }
            if !prefix.suffixes.is_empty() {
                return copy;
            }

            if prefix.letter != "" {
                return copy;
            }
            copy.letter.clear();

            if copy.main.len() > prefix.main.len() {
                copy.main.truncate(prefix.main.len());
            }
            copy
        })();

        &trimed_version == prefix
    }
}

impl FromStr for Version {
    type Err = Error;

    /// Parses `text` into [`Version`].
    ///
    /// There is also the `Version::try_new` method that does exactly the same
    /// thing.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Version::try_new(text)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.main[0])?;
        for v in self.main[1..].iter() {
            write!(f, ".{}", v)?;
        }
        write!(f, "{}", self.letter)?;
        for suffix in self.suffixes.iter() {
            write!(f, "{}", suffix.label)?;
            if !suffix.number.is_empty() {
                write!(f, "{}", suffix.number)?;
            }
        }
        if !self.revision.is_empty() {
            write!(f, "-r{}", self.revision)?;
        }
        Ok(())
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    /// Compares two versions.
    ///
    /// See the following section in PMS for the exact specification:
    /// https://projects.gentoo.org/pms/8/pms.html#x1-260003.3
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare main.
        let major_cmp = compare_generic_number_strings(&self.main[0], &other.main[0]);
        if major_cmp != Ordering::Equal {
            return major_cmp;
        }

        let n = self.main.len().min(other.main.len());
        let post_major_cmp = self.main[1..n]
            .iter()
            .zip(other.main[1..n].iter())
            .map(|(a, b)| compare_post_major_version_strings(a, b))
            .fold(Ordering::Equal, Ordering::then);
        if post_major_cmp != Ordering::Equal {
            return post_major_cmp;
        }

        let main_len_cmp = self.main.len().cmp(&other.main.len());
        if main_len_cmp != Ordering::Equal {
            return main_len_cmp;
        }

        // Compare letter.
        let letter_cmp = self.letter.cmp(&other.letter);
        if letter_cmp != Ordering::Equal {
            return letter_cmp;
        }

        // Compare suffixes.
        let m = self.suffixes.len().min(other.suffixes.len());
        let suffixes_cmp = self.suffixes[..m]
            .iter()
            .zip(other.suffixes[..m].iter())
            .map(|(a, b)| a.cmp(b))
            .fold(Ordering::Equal, Ordering::then);
        if suffixes_cmp != Ordering::Equal {
            return suffixes_cmp;
        }

        if self.suffixes.len() > other.suffixes.len() {
            return if self.suffixes[self.suffixes.len() - 1].label == VersionSuffixLabel::P {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }
        if self.suffixes.len() < other.suffixes.len() {
            return if other.suffixes[other.suffixes.len() - 1].label == VersionSuffixLabel::P {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        compare_generic_number_strings(&self.revision, &other.revision)
    }
}

/// Represents a version suffix, such as "_alpha42".
#[derive(Clone, Debug, Hash)]
pub struct VersionSuffix {
    label: VersionSuffixLabel,
    number: String,
}

impl VersionSuffix {
    /// Returns the label part of the version suffix, e.g. "_alpha".
    pub fn label(&self) -> VersionSuffixLabel {
        self.label
    }

    /// Returns the number part of the version suffix, e.g. "42".
    pub fn number(&self) -> &str {
        &self.number
    }
}

impl Display for VersionSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.label, self.number)
    }
}

impl PartialEq for VersionSuffix {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for VersionSuffix {}

impl PartialOrd for VersionSuffix {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionSuffix {
    fn cmp(&self, other: &Self) -> Ordering {
        let label_cmp = self.label.cmp(&other.label);
        if label_cmp != Ordering::Equal {
            return label_cmp;
        }
        compare_generic_number_strings(&self.number, &other.number)
    }
}

/// Enum for version suffix labels, e.g. "_alpha".
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    strum_macros::AsRefStr,
    strum_macros::Display,
    strum_macros::EnumString,
)]
pub enum VersionSuffixLabel {
    #[strum(serialize = "_alpha")]
    Alpha,
    #[strum(serialize = "_beta")]
    Beta,
    #[strum(serialize = "_pre")]
    Pre,
    #[strum(serialize = "_rc")]
    Rc,
    #[strum(serialize = "_p")]
    P,
}

/// Compares two numerical strings.
fn compare_generic_number_strings(a: &str, b: &str) -> Ordering {
    let a = a.trim_start_matches('0');
    let b = b.trim_start_matches('0');
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }
    a.cmp(b)
}

/// Compares post-major numeric components of two versions.
///
/// See Algorithm 3.3 in PMS for the exact specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-260003.3
fn compare_post_major_version_strings(a: &str, b: &str) -> Ordering {
    if a.starts_with('0') || b.starts_with('0') {
        return a.trim_end_matches('0').cmp(b.trim_end_matches('0'));
    }
    compare_generic_number_strings(a, b)
}

mod parser {
    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{char, digit0, digit1, one_of},
        combinator::{eof, opt},
        multi::many0,
        sequence::preceded,
        IResult,
    };

    fn parse_main(input: &str) -> IResult<&str, Vec<String>> {
        let (input, major) = digit1(input)?;
        let (input, post_major) = many0(preceded(char('.'), digit1))(input)?;
        let mut main = vec![major.to_owned()];
        main.extend(post_major.into_iter().map(|s| s.to_owned()));
        Ok((input, main))
    }

    fn parse_letter(input: &str) -> IResult<&str, String> {
        let (input, letter) = opt(one_of("abcdefghijklmnopqrstuvwxyz"))(input)?;
        let letter = letter.map(|c| c.to_string()).unwrap_or_default();
        Ok((input, letter))
    }

    fn parse_suffix(input: &str) -> IResult<&str, VersionSuffix> {
        let (input, label) = alt((
            tag(VersionSuffixLabel::Alpha.as_ref()),
            tag(VersionSuffixLabel::Beta.as_ref()),
            tag(VersionSuffixLabel::Pre.as_ref()),
            tag(VersionSuffixLabel::Rc.as_ref()),
            tag(VersionSuffixLabel::P.as_ref()),
        ))(input)?;
        let (input, number) = digit0(input)?;
        Ok((
            input,
            VersionSuffix {
                label: label.parse().unwrap(),
                number: number.to_owned(),
            },
        ))
    }

    fn parse_suffixes(input: &str) -> IResult<&str, Vec<VersionSuffix>> {
        many0(parse_suffix)(input)
    }

    fn parse_revision(input: &str) -> IResult<&str, String> {
        let (input, revision) = opt(preceded(tag("-r"), digit1))(input)?;
        let revision = revision.map(|s| s.to_owned()).unwrap_or_default();
        Ok((input, revision))
    }

    pub(super) fn parse_version(input: &str) -> IResult<&str, Version> {
        let (input, main) = parse_main(input)?;
        let (input, letter) = parse_letter(input)?;
        let (input, suffixes) = parse_suffixes(input)?;
        let (input, revision) = parse_revision(input)?;
        let (input, _) = eof(input)?;
        Ok((
            input,
            Version {
                main,
                letter,
                suffixes,
                revision,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_parse_and_to_string() -> Result<()> {
        let cases = [
            "0",
            "1.2.3.4.5.6.7.8",
            "10000000000000000000000",
            "1x",
            "1_alpha",
            "1_alpha42",
            "1_rc_beta3_rc5",
            "1-r0",
            "1-r1000000000000000000",
        ];
        for case in cases {
            let ver = Version::try_new(case)?;
            assert_eq!(ver.to_string(), case);
        }
        Ok(())
    }

    proptest! {
        #[test]
        fn proptest_parse_no_crash(s in "\\PC*") {
            Version::try_new(&s).ok();
        }

        #[test]
        fn proptest_parse_and_to_string(s in VERSION_RE_RAW) {
            let ver = Version::try_new(&s).unwrap();
            assert_eq!(ver.to_string(), s);
        }
    }
}

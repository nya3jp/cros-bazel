// Copyright 2021 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::fmt;
use std::path::Path;

use anyhow::anyhow;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take, take_while, take_while1},
    character::complete::{self, multispace0, multispace1},
    character::is_alphabetic,
    character::is_alphanumeric,
    combinator::{map, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair},
    IResult,
};

use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str, &'a Path>;

/// An enum corresponding to the values that can be assigned to a variable. The two variants
/// correspond to either a literal string or an in-place variable expansion (e.g. "${FOO}").
/// A variable expansion can then recursively contain literal strings and more variable expansions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'a> {
    /// A verbatim section of text, e.g. "foo".
    Literal(Span<'a>),
    /// A variable expansion site, e.g. `${MY_VAR}`.
    Expansion(Span<'a>),
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Literal(s) => write!(f, "{}", s),
            Value::Expansion(name) => write!(f, "${{{}}}", name),
        }
    }
}

/// An [RVal] is the complete expression on the right-hand side of a variable assignment, e.g.
/// `FOO="spam $HAM eggs"` would have an [RVal] of `"spam $HAM eggs"`. In this example, the
/// [RVal] would have the [Value]s of a Literal("spam "), an Expansion{name: "HAM"}, and another
/// Literal(" eggs").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RVal<'a> {
    pub vals: Vec<Value<'a>>,
}

impl<'a> RVal<'a> {
    fn new(vals: Vec<Value<'a>>) -> Self {
        Self { vals }
    }
}

impl fmt::Display for RVal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for val in self.vals.iter() {
            write!(f, "{}", val)?;
        }
        Ok(())
    }
}

/// Represents a statement in a make.conf-like configuration file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<'a> {
    /// An assignment to a variable, e.g. `FOO="bar $baz"`.
    Assign(Span<'a>, RVal<'a>),
    /// Inclusion of another source file, e.g. `source path/to/make.conf`.
    Source(RVal<'a>),
}

/// Represents a make.conf-like configuration file as a sequence of statements.
pub type File<'a> = Vec<Statement<'a>>;

/// Parser entry point for the entirety of a `make.conf` file. Expects the full body of the file
/// as a single [Span] as input.
pub fn full_parse(mut input: Span<'_>, allow_source: bool) -> anyhow::Result<File<'_>> {
    let mut file = File::new();

    // This parser loop re-assigns the remaining text to the `input` variable as fragments
    // are consumed by each sub-parser.
    while !input.is_empty() {
        if let Ok((new_input, _)) = comment_line(input) {
            input = new_input;
            continue;
        }

        if let Ok((new_input, statement)) = assignment(input) {
            file.push(statement);
            input = new_input;
            continue;
        }

        if allow_source {
            if let Ok((new_input, statement)) = source(input) {
                file.push(statement);
                input = new_input;
                continue;
            }
        }

        // Consume any stray leading whitespace, or return an error if we cannot parse further.
        let (new, _) =
            multispace1::<Span, nom::error::VerboseError<Span>>(input).map_err(|_| {
                anyhow!(
                    "Syntax error at line {line_number}:\n\n\
                    {full_line}\n\
                    {caret:>column$}\n\n\
                    Invalid fragment (expected a variable assignment or comment).
                ",
                    line_number = input.location_line(),
                    full_line = std::str::from_utf8(input.get_line_beginning()).unwrap(),
                    caret = '^',
                    column = input.get_column(),
                )
            })?;
        input = new;
    }

    Ok(file)
}

/// Parser to recognize a commented line in a `make.conf` file.
fn comment_line(input: Span) -> IResult<Span, Span> {
    recognize(preceded(complete::char('#'), complete::not_line_ending))(input)
}

/// Parser to recognize a full assignment expression, e.g. `FOO="$BAR $BAZ"`.
fn assignment(input: Span) -> IResult<Span, Statement> {
    map(separated_pair(variable, tag("="), rval), |(lval, rval)| {
        Statement::Assign(lval, rval)
    })(input)
}

/// Parser to recognize a source statement, e.g. `source path/to/make.conf`.
fn source(input: Span) -> IResult<Span, Statement> {
    map(preceded(pair(tag("source"), multispace1), rval), |rval| {
        Statement::Source(rval)
    })(input)
}

/// Parser to recognize a [RVal].
fn rval(input: Span) -> IResult<Span, RVal> {
    alt((double_quoted_rval, single_quoted_rval, unquoted_rval))(input)
}

/// Parser to recognize a properly double-quoted [RVal].
///
/// Spec reference:
/// https://dev.gentoo.org/~ulm/pms/head/pms.html#x1-470005.2.4
///
/// Line continuations are not currently handled properly.
fn double_quoted_rval(input: Span) -> IResult<Span, RVal> {
    map(
        delimited(
            tag("\""),
            many0(alt((double_quoted_literal, escaped_char, expansion))),
            tag("\""),
        ),
        |vals| RVal { vals },
    )(input)
}

/// Parser to recognize a properly single-quoted [RVal].
///
/// Line continuations are not currently handled properly.
fn single_quoted_rval(input: Span) -> IResult<Span, RVal> {
    map(delimited(tag("'"), is_not("'"), tag("'")), |s| RVal {
        vals: vec![Value::Literal(s)],
    })(input)
}

/// Parser to recognize unquoted rvalues, as much as possible.
///
/// These are violations of the PMS, but the ability to correctly parse these is needed to support
/// the few organic usages within the Chrome OS tree.
fn unquoted_rval(input: Span) -> IResult<Span, RVal> {
    let not_ws = |c: char| !c.is_ascii_whitespace();
    // Our best guess for an unquoted rvalue is everything up until the next piece of whitespace.
    let unquoted_literal = map(take_while1(not_ws), Value::Literal);

    map(
        preceded(multispace0, many0(alt((expansion, unquoted_literal)))),
        RVal::new,
    )(input)
}

/// Parser to recognize double-quoted string literals.
fn double_quoted_literal(input: Span<'_>) -> IResult<Span<'_>, Value<'_>> {
    map(is_not("$\"\\"), Value::Literal)(input)
}

/// Parser to recognize double-quoted escaped characters.
fn escaped_char(input: Span<'_>) -> IResult<Span<'_>, Value<'_>> {
    map(preceded(tag("\\"), take(1usize)), Value::Literal)(input)
}

/// Parser to recognize variable names.
fn variable(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    let leading_symbol = |c| is_alphabetic(c as u8) || c == '_';
    let trailing_symbol = |c| is_alphanumeric(c as u8) || c == '_';
    recognize(preceded(
        take_while1(leading_symbol),
        take_while(trailing_symbol),
    ))(input)
}

/// Parser to recognize variable expansions in rvalues.
fn expansion(input: Span<'_>) -> IResult<Span<'_>, Value<'_>> {
    map(
        preceded(
            tag("$"),
            alt((delimited(tag("{"), variable, tag("}")), variable)),
        ),
        Value::Expansion,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use nom::Slice;
    fn null_span(text: &str) -> Span<'_> {
        lazy_static! {
            static ref NULL_PATH: &'static Path = Path::new("");
        }
        Span::new_extra(text, *NULL_PATH)
    }

    const FULL_SAMPLE: &str = r#"# Copyright (c) 2015 The ChromiumOS Authors.
# Distributed under the terms of the GNU General Public License v2

# Settings that are common to all host sdks.  Do not place any board specific
# settings in here, or settings for cross-compiled targets.
#
# See "man 5 make.conf" and "man 5 portage" for the available options.

# Dummy setting so we can use the same append form below.
USE=""

# Various global settings.
USE="${USE} hardened multilib pic pie -introspection -cracklib"

# Custom USE flag ebuilds can use to determine whether it's going into the sdk
# or into a target board.
USE="${USE} cros_host"

# Disable all x11 USE flags for packages within chroot.
USE="${USE} -gtk2 -gtk3 -qt4"

# Enable extended attributes support in our sdk tools.
USE="${USE} xattr"
# But disable using them in the sdk itself for now.
USE="${USE} -filecaps"

# No need to track power in the sdk.
USE="${USE} -power_management"

# We don't boot things inside the sdk.
USE="${USE} -openrc"

# Disable vala inside the sdk
USE="${USE} -vala"

# We only have one rootfs.
USE="${USE} -split-usr"

# Various runtime features that control emerge behavior.
# See "man 5 make.conf" for details.
FEATURES="allow-missing-manifests buildpkg clean-logs -collision-protect
            -ebuild-locks force-mirror -merge-sync -pid-sandbox
            parallel-install -preserve-libs sandbox -strict userfetch
            userpriv usersandbox -unknown-features-warn network-sandbox"

# This is used by profiles/base/profile.bashrc to figure out that we
# are targeting the cros-sdk (in all its various modes).  It should
# be utilized nowhere else!
CROS_SDK_HOST="cros-sdk-host"

# Qemu targets we care about.
QEMU_SOFTMMU_TARGETS="aarch64 arm i386 mips mips64 mips64el mipsel x86_64"
QEMU_USER_TARGETS="aarch64 arm i386 mips mips64 mips64el mipsel x86_64"

# Various compiler defaults.  Should be no arch-specific bits here.
CFLAGS="-O2 -pipe"
LDFLAGS="-Wl,-O2 -Wl,--as-needed"

# We want to migrate away from this at some point.
SYMLINK_LIB="yes"

# Default target(s) for python-r1.eclass
PYTHON_TARGETS="-python2_7 python3_6"
PYTHON_SINGLE_TARGET="-python2_7 python3_6"

# Use clang as the default compiler.
CC="x86_64-pc-linux-gnu-clang"
CXX="x86_64-pc-linux-gnu-clang++"
LD="x86_64-pc-linux-gnu-ld.lld"


    "#;

    #[test]
    fn test_full_example_parse() {
        let res = full_parse(null_span(FULL_SAMPLE), false);
        let file = res.unwrap();
        assert_eq!(file.len(), 22);
    }

    #[test]
    fn test_comment_parse() {
        let res = comment_line(null_span(FULL_SAMPLE));
        let (_, capture) = res.unwrap();

        assert_eq!(
            *capture.fragment(),
            "# Copyright (c) 2015 The ChromiumOS Authors."
        );
    }

    const ASSIGN: &str = r#"USE="${USE} hardened multilib pic pie -introspection -cracklib""#;
    #[test]
    fn test_single_assignment_parse() {
        let span = null_span(ASSIGN);
        let res = full_parse(null_span(ASSIGN), false);
        let file = res.unwrap();

        assert_eq!(
            vec![Statement::Assign(
                span.slice(0..3),
                RVal {
                    vals: vec![
                        Value::Expansion(span.slice(7..10)),
                        Value::Literal(span.slice(11..62))
                    ]
                }
            )],
            file
        );
    }

    const MULTI_ASSIGN: &str = r#"
USE="foo"
USE="${USE} bar"
"#;

    #[test]
    fn test_multi_assignment_parse() {
        let span = null_span(MULTI_ASSIGN);
        let res = full_parse(span, false);
        let file = res.unwrap();

        assert_eq!(
            vec![
                Statement::Assign(
                    span.slice(1usize..4usize),
                    RVal {
                        vals: vec![Value::Literal(span.slice(6usize..9usize))]
                    },
                ),
                Statement::Assign(
                    span.slice(11usize..14usize),
                    RVal {
                        vals: vec![
                            Value::Expansion(span.slice(18usize..21usize)),
                            Value::Literal(span.slice(22usize..26usize)),
                        ]
                    },
                ),
            ],
            file
        );
    }

    const UNQUOTED_STRING: &str = r#"CXX=x86_64-pc-linux-gnu-clang++"#;

    #[test]
    fn test_unquoted_string_parse() {
        let span = null_span(UNQUOTED_STRING);
        let res = full_parse(span, false);
        let file = res.unwrap();

        assert_eq!(
            vec![Statement::Assign(
                span.slice(0usize..3usize),
                RVal {
                    vals: vec![Value::Literal(span.slice(4usize..31usize))]
                },
            ),],
            file
        );
    }

    const ESCAPED_CHARACTERS: &str = r#"FETCHCOMMAND="curl -f -y 30 --retry 9 -L --output \"\${DISTDIR}/\${FILE}\" \"\${URI}\"""#;

    #[test]
    fn test_escaped_characters() {
        let span = null_span(ESCAPED_CHARACTERS);
        let res = full_parse(span, false);
        let file = res.unwrap();
        assert_eq!(
            vec![Statement::Assign(
                span.slice(0usize..12usize),
                RVal {
                    vals: vec![
                        Value::Literal(span.slice(14usize..50usize)),
                        Value::Literal(span.slice(51usize..52usize)),
                        Value::Literal(span.slice(53usize..54usize)),
                        Value::Literal(span.slice(54usize..64usize)),
                        Value::Literal(span.slice(65usize..66usize)),
                        Value::Literal(span.slice(66usize..72usize)),
                        Value::Literal(span.slice(73usize..74usize)),
                        Value::Literal(span.slice(74usize..75usize)),
                        Value::Literal(span.slice(76usize..77usize)),
                        Value::Literal(span.slice(78usize..79usize)),
                        Value::Literal(span.slice(79usize..84usize)),
                        Value::Literal(span.slice(85usize..86usize)),
                    ]
                },
            )],
            file
        );
    }

    const SINGLE_QUOTED_STRING: &str = r#"FETCHCOMMAND_CIPD='/mnt/host/source/chromite/bin/fetch_cipd "\${URI}" "\${DISTDIR}/\${FILE}"'"#;

    #[test]
    fn test_single_quoted_strings() {
        let span = null_span(SINGLE_QUOTED_STRING);
        let res = full_parse(span, false);
        let file = res.unwrap();
        assert_eq!(
            vec![Statement::Assign(
                span.slice(0usize..17usize),
                RVal {
                    vals: vec![Value::Literal(span.slice(19usize..92usize))]
                },
            )],
            file
        );
    }

    const SOURCE_FILE: &str = r#"source path/to/make.conf"#;

    #[test]
    fn test_source_file_disabled() {
        let span = null_span(SOURCE_FILE);
        full_parse(span, false).expect_err("full_parse should fail");
    }

    #[test]
    fn test_source_file_enabled() {
        let span = null_span(SOURCE_FILE);
        let res = full_parse(span, true);
        let file = res.unwrap();
        assert_eq!(
            vec![Statement::Source(RVal {
                vals: vec![Value::Literal(span.slice(7usize..24usize))]
            },)],
            file
        );
    }
}

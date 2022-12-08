// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod algorithm;
pub mod package;
pub(self) mod parser;
pub mod uri;

use std::{convert::Infallible, fmt::Display, str::FromStr};

use rayon::prelude::*;

use crate::data::UseMap;

use self::parser::{DependencyParser, DependencyParserType};

/// Trait for a matcher function.
pub trait Predicate<T: ?Sized> {
    fn matches(&self, target: &T) -> bool;
}

/// Generic dependency expression.
///
/// The generic type argument `L` stands for the "leaf" type, with which you
/// can reuse the type for different dependency types, such as package
/// dependencies and URI dependencies.
///
/// See the following section in the PMS for the detailed specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-730008.2
#[derive(Clone, Debug)]
pub enum Dependency<L> {
    /// Leaf dependency that is specific to the actual dependency type.
    Leaf(L),
    /// Dependency compositing zero or more dependencies recursively, such as
    /// all-of and any-of.
    Composite(Box<CompositeDependency<L>>),
}

/// Composite dependency expression that contains zero or more dependencies
/// recursively.
#[derive(Clone, Debug)]
pub enum CompositeDependency<L> {
    /// All-of dependencies: satisfied when all of child dependencies are
    /// satisfied.
    /// If an all-of dependency has no child, it is considered constant true.
    AllOf { children: Vec<Dependency<L>> },
    /// Any-of dependencies: satisfied when any one of child dependencies are
    /// satisfied.
    /// If an any-of dependency has no child, it is considered constant false.
    AnyOf { children: Vec<Dependency<L>> },
    /// USE conditional dependencies: the child dependency is evaluated only
    /// when a certain USE flag has an expected value.
    UseConditional {
        name: String,
        expect: bool,
        child: Dependency<L>,
    },
}

impl<L> Dependency<L> {
    pub fn new_composite(composite: CompositeDependency<L>) -> Self {
        Self::Composite(Box::new(composite))
    }

    /// Creates a dependency expression representing a constant boolean.
    pub fn new_constant(b: bool) -> Self {
        if b {
            Self::new_composite(CompositeDependency::AllOf {
                children: Vec::new(),
            })
        } else {
            Self::new_composite(CompositeDependency::AnyOf {
                children: Vec::new(),
            })
        }
    }

    /// Checks if a dependency expression represents a constant boolean.
    pub fn is_constant(&self) -> Option<bool> {
        match self {
            Self::Composite(composite) => match &**composite {
                CompositeDependency::AllOf { children } if children.is_empty() => Some(true),
                CompositeDependency::AnyOf { children } if children.is_empty() => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

impl<L> Dependency<L> {
    pub fn map_tree(self, mut f: impl FnMut(Self) -> Self) -> Self {
        self.try_map_tree(move |d| Result::<Self, Infallible>::Ok(f(d)))
            .unwrap()
    }

    pub fn try_map_tree<E>(self, mut f: impl FnMut(Self) -> Result<Self, E>) -> Result<Self, E> {
        self.try_map_tree_impl(&mut f)
    }

    fn try_map_tree_impl<E>(self, f: &mut impl FnMut(Self) -> Result<Self, E>) -> Result<Self, E> {
        let tree = match self {
            Self::Composite(composite) => Self::new_composite(match *composite {
                CompositeDependency::AllOf { children } => CompositeDependency::AllOf {
                    children: children
                        .into_iter()
                        .map(|child| child.try_map_tree_impl(f))
                        .collect::<Result<Vec<_>, E>>()?,
                },
                CompositeDependency::AnyOf { children } => CompositeDependency::AnyOf {
                    children: children
                        .into_iter()
                        .map(|child| child.try_map_tree_impl(f))
                        .collect::<Result<Vec<_>, E>>()?,
                },
                CompositeDependency::UseConditional {
                    name,
                    expect,
                    child,
                } => CompositeDependency::UseConditional {
                    name,
                    expect,
                    child: child.try_map_tree_impl(f)?,
                },
            }),
            Self::Leaf(leaf) => Self::Leaf(leaf),
        };
        f(tree)
    }
}

impl<L: Send> Dependency<L> {
    pub fn map_tree_par(self, f: impl Fn(Self) -> Self + Send + Sync) -> Self {
        self.try_map_tree_par(move |d| Result::<Self, Infallible>::Ok(f(d)))
            .unwrap()
    }

    pub fn try_map_tree_par<E: Send>(
        self,
        f: impl Fn(Self) -> Result<Self, E> + Sync,
    ) -> Result<Self, E> {
        self.try_map_tree_par_impl(&f)
    }

    fn try_map_tree_par_impl<E: Send>(
        self,
        f: &(impl Fn(Self) -> Result<Self, E> + Sync),
    ) -> Result<Self, E> {
        f(match self {
            Self::Composite(composite) => Self::new_composite(match *composite {
                CompositeDependency::AllOf { children } => CompositeDependency::AllOf {
                    children: children
                        .into_par_iter()
                        .map(|child| child.try_map_tree_par_impl(f))
                        .collect::<Result<Vec<_>, E>>()?,
                },
                CompositeDependency::AnyOf { children } => CompositeDependency::AnyOf {
                    children: children
                        .into_par_iter()
                        .map(|child| child.try_map_tree_par_impl(f))
                        .collect::<Result<Vec<_>, E>>()?,
                },
                CompositeDependency::UseConditional {
                    name,
                    expect,
                    child,
                } => CompositeDependency::UseConditional {
                    name,
                    expect,
                    child: child.try_map_tree_par_impl(f)?,
                },
            }),
            Self::Leaf(leaf) => Self::Leaf(leaf),
        })
    }
}

impl<L: Display> Display for Dependency<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Leaf(leaf) => leaf.fmt(f),
            Self::Composite(composite) => match &**composite {
                CompositeDependency::AllOf { children } => {
                    write!(f, "(")?;
                    for child in children.iter() {
                        write!(f, " {}", child)?;
                    }
                    write!(f, " )")
                }
                CompositeDependency::AnyOf { children } => {
                    write!(f, "|| (")?;
                    for child in children.iter() {
                        write!(f, " {}", child)?;
                    }
                    write!(f, " )")
                }
                CompositeDependency::UseConditional {
                    name,
                    expect,
                    child,
                } => {
                    if !expect {
                        write!(f, "!")?;
                    }
                    write!(f, "{}? {}", name, child)
                }
            },
        }
    }
}

impl<T: AsRef<UseMap>, L: Predicate<T>> Predicate<T> for Dependency<L> {
    fn matches(&self, target: &T) -> bool {
        match self {
            Self::Leaf(leaf) => leaf.matches(target),
            Self::Composite(composite) => {
                match &**composite {
                    CompositeDependency::AllOf { children } => {
                        children.iter().all(|child| child.matches(target))
                    }
                    CompositeDependency::AnyOf { children } => {
                        children.iter().any(|child| child.matches(target))
                    }
                    CompositeDependency::UseConditional {
                        name,
                        expect,
                        child,
                    } => {
                        let use_map = target.as_ref();
                        let value = use_map.get(name);
                        // Assume that a USE flag is unset when it is not declared in IUSE.
                        let value = *value.unwrap_or(&false);
                        if value != *expect {
                            // TODO: Ignore the expression rather than returning true.
                            // The current behavior does not match Portage's.
                            // For example, "|| ( foo? ( a/b ) )" should be considered
                            // unsatisfiable when foo is unset.
                            true
                        } else {
                            child.matches(target)
                        }
                    }
                }
            }
        }
    }
}

impl<L: DependencyParserType<L>> FromStr for Dependency<L> {
    type Err = <<L as DependencyParserType<L>>::Parser as DependencyParser<Dependency<L>>>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        L::Parser::parse(s)
    }
}

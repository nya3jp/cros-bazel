// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod algorithm;
pub mod package;
mod parser;
pub mod requse;
pub mod restrict;
pub mod uri;

use std::{convert::Infallible, fmt::Display, str::FromStr};

use anyhow::Result;
use itertools::Itertools;
use rayon::prelude::*;

use crate::data::UseMap;

use self::parser::DependencyParser;

/// General-purpose predicate with two-valued logic.
///
/// Leaf dependencies should implement this logic so that they can be used with
/// [`Dependency`].
pub trait Predicate<T: ?Sized> {
    fn matches(&self, source_use_map: &UseMap, target: &T) -> Result<bool>;
}

/// Similar to [`Predicate`], but uses three-valued logic.
///
/// Composite dependencies return three-valued logic. When a dependency
/// returns [`None`], it should be treated as if the dependency does not
/// exist from the first place.
///
/// Notably, USE-conditional dependency returns [`None`] when its USE flag
/// precondition is not satisfied.
///
/// For example, `|| ( foo? ( a/b ) )` should be considered unsatisfiable
/// when `foo` is unset.
pub trait ThreeValuedPredicate<T: ?Sized> {
    fn matches(&self, source_use_map: &UseMap, target: &T) -> Result<Option<bool>>;
}

/// A bundle of types needed to instantiate [`Dependency`].
pub trait DependencyMeta: Clone + std::fmt::Debug + Eq {
    /// The type of leaf elements of the dependency type.
    type Leaf: Clone + std::fmt::Debug + Eq;

    /// The type of the parser producing the dependency type.
    type Parser: DependencyParser;
}

/// Generic dependency expression.
///
/// See the following section in the PMS for the detailed specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-730008.2
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Dependency<M: DependencyMeta> {
    /// Leaf dependency that is specific to the actual dependency type.
    Leaf(M::Leaf),
    /// Dependency compositing zero or more dependencies recursively, such as
    /// all-of and any-of.
    Composite(Box<CompositeDependency<Self>>),
}

/// Composite dependency expression that contains zero or more dependencies
/// recursively.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompositeDependency<D> {
    /// All-of dependencies: satisfied when all of child dependencies are
    /// satisfied.
    /// If an all-of dependency has no child, it is considered constant true,
    /// but prefer `Constant` because it can also carry a debug message.
    AllOf { children: Vec<D> },
    /// Any-of dependencies: satisfied when any one of child dependencies are
    /// satisfied.
    /// If an any-of dependency has no child, it is considered constant false,
    /// but prefer `Constant` because it can also carry a debug message.
    AnyOf { children: Vec<D> },
    /// USE conditional dependencies: the child dependencies are evaluated only
    /// when a certain USE flag has an expected value.
    UseConditional {
        name: String,
        expect: bool,
        /// Child dependencies are interpreted in the same way as all-of.
        children: Vec<D>,
    },
    /// The constant value with a reason for debugging. This is preferred over
    /// `AllOf`/`AnyOf` with no children for better debuggability.
    Constant { value: bool, reason: String },
}

impl<M: DependencyMeta> Dependency<M> {
    pub fn new_composite(composite: CompositeDependency<Self>) -> Self {
        Self::Composite(Box::new(composite))
    }

    /// Creates a dependency expression representing a constant boolean.
    pub fn new_constant(value: bool, reason: &str) -> Self {
        Self::new_composite(CompositeDependency::Constant {
            value,
            reason: reason.to_owned(),
        })
    }

    /// Checks if a dependency expression represents a constant boolean.
    ///
    /// If it is a constant, returns a pair of the constant value and a message
    /// describing why it is evaluated to a constant value.
    pub fn check_constant(&self) -> Option<(bool, &str)> {
        match self {
            Self::Composite(composite) => match &**composite {
                CompositeDependency::AllOf { children } if children.is_empty() => {
                    Some((true, "all-of () dependency is empty"))
                }
                CompositeDependency::AnyOf { children } if children.is_empty() => {
                    Some((false, "any-of || dependency is empty"))
                }
                CompositeDependency::Constant { value, reason } => Some((*value, reason.as_str())),
                _ => None,
            },
            _ => None,
        }
    }
}

impl<M: DependencyMeta> Dependency<M> {
    pub fn map_tree(self, mut f: impl FnMut(Self) -> Self) -> Self {
        self.try_map_tree(move |d| Result::<Self, Infallible>::Ok(f(d)))
            .unwrap()
    }

    pub fn try_map_tree<E>(self, mut f: impl FnMut(Self) -> Result<Self, E>) -> Result<Self, E> {
        Ok(self.try_flat_map_tree(move |d| Ok(Some(f(d)?)))?.unwrap())
    }

    // TODO: Support [`Iterator`] in general, instead of [`Option`] only.
    pub fn flat_map_tree(self, mut f: impl FnMut(Self) -> Option<Self>) -> Option<Self> {
        self.try_flat_map_tree(move |d| Result::<Option<Self>, Infallible>::Ok(f(d)))
            .unwrap()
    }

    // TODO: Support [`Iterator`] in general, instead of [`Option`] only.
    pub fn try_flat_map_tree<E>(
        self,
        mut f: impl FnMut(Self) -> Result<Option<Self>, E>,
    ) -> Result<Option<Self>, E> {
        self.try_flat_map_tree_impl(&mut f)
    }

    fn try_flat_map_tree_impl<E>(
        self,
        f: &mut impl FnMut(Self) -> Result<Option<Self>, E>,
    ) -> Result<Option<Self>, E> {
        let tree = match self {
            Self::Composite(composite) => Self::new_composite(match *composite {
                CompositeDependency::AllOf { children } => CompositeDependency::AllOf {
                    children: children
                        .into_iter()
                        .map(|child| child.try_flat_map_tree_impl(f))
                        .flatten_ok()
                        .collect::<Result<Vec<_>, E>>()?,
                },
                CompositeDependency::AnyOf { children } => CompositeDependency::AnyOf {
                    children: children
                        .into_iter()
                        .map(|child| child.try_flat_map_tree_impl(f))
                        .flatten_ok()
                        .collect::<Result<Vec<_>, E>>()?,
                },
                CompositeDependency::UseConditional {
                    name,
                    expect,
                    children,
                } => CompositeDependency::UseConditional {
                    name,
                    expect,
                    children: children
                        .into_iter()
                        .map(|child| child.try_flat_map_tree_impl(f))
                        .flatten_ok()
                        .collect::<Result<Vec<_>, E>>()?,
                },
                constant @ CompositeDependency::Constant { .. } => constant,
            }),
            leaf @ Self::Leaf(_) => leaf,
        };
        f(tree)
    }
}

impl<M: DependencyMeta> Dependency<M>
where
    M::Leaf: Send,
{
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
                    children,
                } => CompositeDependency::UseConditional {
                    name,
                    expect,
                    children: children
                        .into_par_iter()
                        .map(|child| child.try_map_tree_par_impl(f))
                        .collect::<Result<Vec<_>, E>>()?,
                },
                constant @ CompositeDependency::Constant { .. } => constant,
            }),
            leaf @ Self::Leaf(_) => leaf,
        })
    }
}

impl<M: DependencyMeta> Default for Dependency<M> {
    /// The default value of [`Dependency`] is the constant true.
    fn default() -> Self {
        Self::new_constant(true, "No dependency")
    }
}

impl<M: DependencyMeta> Display for Dependency<M>
where
    M::Leaf: Display,
{
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
                    children,
                } => {
                    if !expect {
                        write!(f, "!")?;
                    }
                    write!(f, "{}? (", name)?;
                    for child in children.iter() {
                        write!(f, " {}", child)?;
                    }
                    write!(f, " )")
                }
                CompositeDependency::Constant { value, .. } => {
                    if *value {
                        write!(f, "( )")
                    } else {
                        write!(f, "|| ( )")
                    }
                }
            },
        }
    }
}

impl<M: DependencyMeta, T> ThreeValuedPredicate<T> for Dependency<M>
where
    M::Leaf: Predicate<T>,
{
    fn matches(&self, source_use_map: &UseMap, target: &T) -> Result<Option<bool>> {
        match self {
            Self::Leaf(leaf) => Ok(Some(leaf.matches(source_use_map, target)?)),
            Self::Composite(composite) => {
                match &**composite {
                    CompositeDependency::AllOf { children } => Ok(Some(
                        children
                            .iter()
                            .map(|child| child.matches(source_use_map, target))
                            .collect::<Result<Vec<_>>>()?
                            .into_iter()
                            .all(|b| b != Some(false)),
                    )),
                    CompositeDependency::AnyOf { children } => Ok(Some(
                        children
                            .iter()
                            .map(|child| child.matches(source_use_map, target))
                            .collect::<Result<Vec<_>>>()?
                            .into_iter()
                            .any(|b| b == Some(true)),
                    )),
                    CompositeDependency::UseConditional {
                        name,
                        expect,
                        children,
                    } => {
                        let value = source_use_map.get(name);
                        // Assume that a USE flag is unset when it is not declared in IUSE.
                        let value = *value.unwrap_or(&false);
                        if value != *expect {
                            Ok(None)
                        } else {
                            Ok(Some(
                                children
                                    .iter()
                                    .map(|child| child.matches(source_use_map, target))
                                    .collect::<Result<Vec<_>>>()?
                                    .into_iter()
                                    .all(|b| b != Some(false)),
                            ))
                        }
                    }
                    CompositeDependency::Constant { value, .. } => Ok(Some(*value)),
                }
            }
        }
    }
}

impl<M: DependencyMeta> FromStr for Dependency<M>
where
    M::Parser: DependencyParser<Output = Self>,
{
    type Err = <M::Parser as DependencyParser>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        M::Parser::parse(s)
    }
}

/// Generic dependency expression.
///
/// This is similar to [`Dependency`], but it supports additional composite
/// dependencies, such as exactly-one-of and at-most-one-of.
///
/// See the following section in the PMS for the detailed specification:
/// https://projects.gentoo.org/pms/8/pms.html#x1-730008.2
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComplexDependency<M: DependencyMeta> {
    /// Leaf dependency that is specific to the actual dependency type.
    Leaf(M::Leaf),
    /// Dependency compositing zero or more dependencies recursively, such as
    /// all-of and any-of.
    Composite(Box<ComplexCompositeDependency<Self>>),
}

impl<M: DependencyMeta> ComplexDependency<M> {
    pub fn new_composite(composite: ComplexCompositeDependency<Self>) -> Self {
        Self::Composite(Box::new(composite))
    }
}

impl<M: DependencyMeta> Display for ComplexDependency<M>
where
    M::Leaf: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Leaf(leaf) => leaf.fmt(f),
            Self::Composite(composite) => match &**composite {
                ComplexCompositeDependency::AllOf { children } => {
                    write!(f, "(")?;
                    for child in children.iter() {
                        write!(f, " {}", child)?;
                    }
                    write!(f, " )")
                }
                ComplexCompositeDependency::AnyOf { children } => {
                    write!(f, "|| (")?;
                    for child in children.iter() {
                        write!(f, " {}", child)?;
                    }
                    write!(f, " )")
                }
                ComplexCompositeDependency::ExactlyOneOf { children } => {
                    write!(f, "^^ (")?;
                    for child in children.iter() {
                        write!(f, " {}", child)?;
                    }
                    write!(f, " )")
                }
                ComplexCompositeDependency::AtMostOneOf { children } => {
                    write!(f, "?? (")?;
                    for child in children.iter() {
                        write!(f, " {}", child)?;
                    }
                    write!(f, " )")
                }
                ComplexCompositeDependency::UseConditional {
                    name,
                    expect,
                    children,
                } => {
                    if !expect {
                        write!(f, "!")?;
                    }
                    write!(f, "{}? (", name)?;
                    for child in children.iter() {
                        write!(f, " {}", child)?;
                    }
                    write!(f, " )")
                }
            },
        }
    }
}

/// Composite dependency expression that contains zero or more dependencies
/// recursively.
///
/// This is similar to [`CompositeDependency`], but it supports additional
/// composite dependencies, such as exactly-one-of and at-most-one-of.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComplexCompositeDependency<D> {
    /// All-of dependencies: satisfied when all of child dependencies are
    /// satisfied.
    AllOf { children: Vec<D> },
    /// Any-of dependencies: satisfied when any one of child dependencies are
    /// satisfied.
    AnyOf { children: Vec<D> },
    /// Exactly-one-of dependencies: satisfied when exactly one of child
    /// dependencies is satisfied.
    ExactlyOneOf { children: Vec<D> },
    /// At-most-one-of dependencies: satisfied when at most one of child
    /// dependencies are satisfied.
    AtMostOneOf { children: Vec<D> },
    /// USE conditional dependencies: the child dependencies are evaluated only
    /// when a certain USE flag has an expected value.
    UseConditional {
        name: String,
        expect: bool,
        /// Child dependencies are interpreted in the same way as all-of.
        children: Vec<D>,
    },
}

impl<M: DependencyMeta, T> ThreeValuedPredicate<T> for ComplexDependency<M>
where
    M::Leaf: Predicate<T>,
{
    fn matches(&self, source_use_map: &UseMap, target: &T) -> Result<Option<bool>> {
        match self {
            Self::Leaf(leaf) => Ok(Some(leaf.matches(source_use_map, target)?)),
            Self::Composite(composite) => {
                match &**composite {
                    ComplexCompositeDependency::AllOf { children } => Ok(Some(
                        children
                            .iter()
                            .map(|child| child.matches(source_use_map, target))
                            .collect::<Result<Vec<_>>>()?
                            .into_iter()
                            .all(|b| b != Some(false)),
                    )),
                    ComplexCompositeDependency::AnyOf { children } => Ok(Some(
                        children
                            .iter()
                            .map(|child| child.matches(source_use_map, target))
                            .collect::<Result<Vec<_>>>()?
                            .into_iter()
                            .any(|b| b == Some(true)),
                    )),
                    ComplexCompositeDependency::ExactlyOneOf { children } => Ok(Some(
                        children
                            .iter()
                            .map(|child| child.matches(source_use_map, target))
                            .collect::<Result<Vec<_>>>()?
                            .into_iter()
                            .filter(|b| *b == Some(true))
                            .count()
                            == 1,
                    )),
                    ComplexCompositeDependency::AtMostOneOf { children } => Ok(Some(
                        children
                            .iter()
                            .map(|child| child.matches(source_use_map, target))
                            .collect::<Result<Vec<_>>>()?
                            .into_iter()
                            .filter(|b| *b == Some(true))
                            .count()
                            <= 1,
                    )),
                    ComplexCompositeDependency::UseConditional {
                        name,
                        expect,
                        children,
                    } => {
                        let value = source_use_map.get(name);
                        // Assume that a USE flag is unset when it is not declared in IUSE.
                        let value = *value.unwrap_or(&false);
                        if value != *expect {
                            Ok(None)
                        } else {
                            Ok(Some(
                                children
                                    .iter()
                                    .map(|child| child.matches(source_use_map, target))
                                    .collect::<Result<Vec<_>>>()?
                                    .into_iter()
                                    .all(|b| b != Some(false)),
                            ))
                        }
                    }
                }
            }
        }
    }
}

impl<M: DependencyMeta> FromStr for ComplexDependency<M>
where
    M::Parser: DependencyParser<Output = Self>,
{
    type Err = <M::Parser as DependencyParser>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        M::Parser::parse(s)
    }
}

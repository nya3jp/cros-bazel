// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use std::{
    collections::HashMap,
    fmt::Debug,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use crate::data::Vars;

use super::{ConfigNode, ConfigNodeValue, ConfigSource};

pub mod generate;
mod parser;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Value {
    Literal(String),
    UnresolvedExpansion(String),
}

impl Value {
    fn fmt_with_env(&self, mut w: impl std::fmt::Write, env: &Vars) {
        match self {
            Value::Literal(s) => w.write_str(s.as_ref()).unwrap(),
            Value::UnresolvedExpansion(name) => w
                .write_str(env.get(name).map(|s| &**s).unwrap_or_default())
                .unwrap(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RVal {
    vals: Vec<Value>,
}

impl RVal {
    pub fn new() -> Self {
        Self { vals: Vec::new() }
    }

    pub fn push(&mut self, v: Value) {
        match self.vals.last_mut() {
            None => {
                self.vals.push(v);
            }
            Some(last) => match (last, &v) {
                (Value::Literal(a), Value::Literal(b)) => {
                    a.push_str(b);
                }
                _ => {
                    self.vals.push(v);
                }
            },
        }
    }

    pub fn evaluate(&self, env: &Vars) -> String {
        let mut s = String::new();
        self.fmt_with_env(&mut s, env);
        s
    }

    fn fmt_with_env(&self, mut w: impl std::fmt::Write, env: &Vars) {
        for value in self.vals.iter() {
            value.fmt_with_env(&mut w, env);
        }
    }

    pub fn try_to_string_no_unresolved_expansion(&self) -> Result<String> {
        let mut result = String::new();
        for value in self.vals.iter() {
            match value {
                Value::Literal(s) => {
                    result.push_str(s);
                }
                Value::UnresolvedExpansion(name) => {
                    bail!("contains unresolved expansion ${}", name);
                }
            }
        }
        Ok(result)
    }
}

impl FromIterator<Value> for RVal {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Self {
            vals: Vec::from_iter(iter),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MakeConf {
    sources: Vec<PathBuf>,
    values: HashMap<String, RVal>,
}

impl MakeConf {
    pub fn new_for_testing(sources: Vec<PathBuf>, values: HashMap<&str, &str>) -> Self {
        let mut rvalues: HashMap<String, RVal> = HashMap::new();
        for (name, value) in values {
            let rval = RVal::from_iter([Value::Literal(value.to_owned())]);
            rvalues.insert(name.to_string(), rval);
        }
        Self {
            sources,
            values: rvalues,
        }
    }

    pub fn load(
        path: &Path,
        base_dir: &Path,
        allow_source: bool,
        allow_missing: bool,
    ) -> Result<Self> {
        let mut conf = Self {
            sources: Vec::new(),
            values: HashMap::new(),
        };
        conf.load_file(path, base_dir, allow_source, allow_missing)?;
        Ok(conf)
    }

    fn load_file(
        &mut self,
        path: &Path,
        base_dir: &Path,
        allow_source: bool,
        allow_missing: bool,
    ) -> Result<()> {
        let source = base_dir.join(path);
        let context = || format!("Failed to load {}", source.display());

        if allow_missing && !source.exists() {
            return Ok(());
        }
        if source.is_dir() {
            let mut names = Vec::new();
            for entry in source.read_dir()? {
                names.push(entry?.file_name());
            }
            names.sort();

            for name in names {
                let new_path = path.join(name);
                self.load_file(&new_path, base_dir, allow_source, allow_missing)
                    .with_context(context)?;
            }
            return Ok(());
        }

        let content = read_to_string(&source).with_context(context)?;
        let span = parser::Span::new_extra(&content, &source);
        let statements = parser::full_parse(span, allow_source).with_context(context)?;

        // Resolves [parser::RVal] into [RVal].
        let evaluate_parser_rval = |values: &HashMap<String, RVal>, rval: parser::RVal| {
            let mut resolved_rval = RVal::new();
            for value in rval.vals {
                match value {
                    parser::Value::Literal(s) => {
                        let s = *s.fragment();
                        resolved_rval.push(Value::Literal(s.to_owned()));
                    }
                    parser::Value::Expansion(name) => {
                        let name = *name.fragment();
                        match values.get(name) {
                            None => {
                                resolved_rval.push(Value::UnresolvedExpansion(name.to_owned()));
                            }
                            Some(expanded_rval) => {
                                for value in expanded_rval.vals.iter() {
                                    resolved_rval.push(value.clone());
                                }
                            }
                        }
                    }
                }
            }
            resolved_rval
        };

        for statement in statements {
            match statement {
                parser::Statement::Assign(lval, rval) => {
                    self.values.insert(
                        (*lval.fragment()).to_owned(),
                        evaluate_parser_rval(&self.values, rval),
                    );
                }
                parser::Statement::Source(rval) => {
                    let rval = evaluate_parser_rval(&self.values, rval);
                    let source_path = base_dir.join(
                        rval.try_to_string_no_unresolved_expansion()
                            .with_context(context)?,
                    );
                    self.load_file(&source_path, base_dir, allow_source, allow_missing)
                        .with_context(context)?;
                }
            }
        }

        self.sources.push(source);

        Ok(())
    }
}

impl ConfigSource for MakeConf {
    fn evaluate_configs(&self, env: &mut Vars) -> Vec<ConfigNode> {
        // Evaluate variables.
        let mut vars = Vars::new();
        for (name, rval) in self.values.iter() {
            vars.insert(name.to_owned(), rval.evaluate(env));
        }

        // Update `env` with computed variables.
        env.extend(vars.clone());

        vec![ConfigNode {
            sources: self.sources.clone(),
            value: ConfigNodeValue::Vars(vars),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::write_files;

    const MANY_ASSIGN: &str = r#"
USE="foo"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
"#;

    #[test]
    fn test_many_assign_evaluation() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(dir, [("make.conf", MANY_ASSIGN)])?;
        let conf = MakeConf::load(&PathBuf::from("make.conf"), dir, false, false)?;

        assert_eq!(
            HashMap::from_iter([(
                "USE".to_owned(),
                RVal::from_iter([Value::Literal(
                    "foo bar bar bar bar bar bar bar bar bar".to_owned()
                )])
            )]),
            conf.values
        );
        Ok(())
    }

    const TWENTY_FIVE_LAUGHS: &str = r#"
LOL="lol"
LOL="${LOL} ${LOL} ${LOL} ${LOL} ${LOL}"
LOL="${LOL} ${LOL} ${LOL} ${LOL} ${LOL}"
"#;

    const TWENTY_FIVE_LAUGHS_EXPANDED: &str = "lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol";

    #[test]
    fn test_25_laughs_evaluation() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(dir, [("make.conf", TWENTY_FIVE_LAUGHS)])?;
        let conf = MakeConf::load(&PathBuf::from("make.conf"), dir, false, false)?;

        assert_eq!(
            HashMap::from_iter([(
                "LOL".to_owned(),
                RVal::from_iter([Value::Literal(TWENTY_FIVE_LAUGHS_EXPANDED.to_owned())])
            )]),
            conf.values
        );
        Ok(())
    }

    #[test]
    fn test_unresolved_expansion() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [(
                "make.conf",
                r#"
                    USE="${USE} foo"
                    USE="${USE} bar"
                "#,
            )],
        )?;
        let conf = MakeConf::load(&PathBuf::from("make.conf"), dir, false, false)?;

        assert_eq!(
            HashMap::from_iter([(
                "USE".to_owned(),
                RVal::from_iter([
                    Value::UnresolvedExpansion("USE".to_owned()),
                    Value::Literal(" foo bar".to_owned()),
                ])
            )]),
            conf.values
        );
        Ok(())
    }

    fn write_source_files(dir: &Path) -> Result<()> {
        write_files(
            dir,
            [
                (
                    "make.conf",
                    r#"
                        USE="$USE a"
                        source make.conf.user
                        USE="$USE b"
                        source make.conf.user
                        USE="$USE c"
                    "#,
                ),
                (
                    "make.conf.user",
                    r#"
                        USE="$USE x"
                    "#,
                ),
            ],
        )
    }

    #[test]
    fn test_directory() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                ("make.conf/a.conf", "USE=\"$USE a\""),
                ("make.conf/b.conf", "USE=\"$USE b\""),
                ("make.conf/c.conf", "USE=\"$USE c\""),
            ],
        )?;

        let conf = MakeConf::load(&PathBuf::from("make.conf"), dir, false, false)?;

        assert_eq!(
            HashMap::from_iter([(
                "USE".to_owned(),
                RVal::from_iter([
                    Value::UnresolvedExpansion("USE".to_owned()),
                    Value::Literal(" a b c".to_owned()),
                ])
            )]),
            conf.values
        );
        Ok(())
    }

    #[test]
    fn test_allow_source_disabled() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_source_files(dir)?;
        MakeConf::load(&PathBuf::from("make.conf"), dir, false, false)
            .expect_err("MakeConf::load should fail");
        Ok(())
    }

    #[test]
    fn test_allow_source_enabled() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_source_files(dir)?;
        let conf = MakeConf::load(&PathBuf::from("make.conf"), dir, true, false)?;

        assert_eq!(
            HashMap::from_iter([(
                "USE".to_owned(),
                RVal::from_iter([
                    Value::UnresolvedExpansion("USE".to_owned()),
                    Value::Literal(" a x b x c".to_owned()),
                ])
            )]),
            conf.values
        );

        assert_eq!(
            vec![
                dir.join("make.conf.user"),
                dir.join("make.conf.user"),
                dir.join("make.conf")
            ],
            conf.sources
        );
        Ok(())
    }

    #[test]
    fn test_allow_missing_disabled() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        MakeConf::load(&PathBuf::from("make.conf"), dir, false, false)
            .expect_err("MakeConf::load() did not fail");
        Ok(())
    }

    #[test]
    fn test_allow_missing_enabled() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        MakeConf::load(&PathBuf::from("make.conf"), dir, false, true)?;
        Ok(())
    }
}

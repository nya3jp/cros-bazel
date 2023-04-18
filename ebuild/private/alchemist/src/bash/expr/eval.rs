// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::bash::expr::AndOrListItem::AndOp;
use crate::bash::expr::AndOrListItem::OrOp;
use crate::data::UseMap;

use anyhow::bail;
use anyhow::Result;

use super::AndOrList;
use super::BashExpr;
use super::SimpleCommand;

fn eval_use_command(args: &[String], map: &UseMap) -> Result<bool> {
    if args.len() != 1 {
        bail!("Usage: use [!]<flag>")
    }

    let mut flag = args[0].as_str();
    let mut expect = true;
    if flag.starts_with('!') {
        expect = false;
        flag = &flag[1..];
    }

    let value = *map.get(flag).unwrap_or(&false);
    Ok(expect == value)
}

fn eval_command(cmd: &SimpleCommand, map: &UseMap) -> Result<bool> {
    Ok(match cmd.tokens.split_first() {
        Some((cmd, args)) => match cmd.as_str() {
            "true" => true,
            "false" => false,
            "use" => eval_use_command(args, map)?,
            &_ => bail!("Unknown command '{}'", cmd),
        },
        None => bail!("Empty command"),
    })
}

fn eval_and_or_list(list: &AndOrList, map: &UseMap) -> Result<bool> {
    let mut value = eval_command(&list.initial, map)?;

    for op in &list.ops {
        match op {
            AndOp(cmd) => {
                if value {
                    value = eval_command(cmd, map)?;
                } else {
                    continue;
                }
            }
            OrOp(cmd) => {
                if !value {
                    value = eval_command(cmd, map)?;
                } else {
                    continue;
                }
            }
        }
    }

    Ok(value)
}

pub(super) fn eval(expr: &BashExpr, map: &UseMap) -> Result<bool> {
    eval_and_or_list(&expr.and_or_list, map)
}

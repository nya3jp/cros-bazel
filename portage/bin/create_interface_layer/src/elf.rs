// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use anyhow::{Context, Result};
use elf::{
    abi::{SHN_ABS, STB_LOCAL, STT_NOTYPE, STT_OBJECT, VER_FLG_BASE},
    endian::AnyEndian,
    ElfStream,
};
use std::path::Path;

#[derive(Debug, Eq, PartialEq)]
struct Symbol {
    name: String,
    version: Option<String>,
    hidden: bool,
}

/// Checks if the symbol is external and available for linking
fn is_exported_symbol(symbol: &elf::symbol::Symbol) -> bool {
    // Here is an example .dynsym table.
    //
    // Symbol table '.dynsym' contains 12 entries:
    //    Num:    Value          Size Type    Bind   Vis      Ndx Name
    //      0: 0000000000000000     0 NOTYPE  LOCAL  DEFAULT  UND
    //      1: 0000000000000000     0 NOTYPE  WEAK   DEFAULT  UND __gmon_start__
    //      2: 0000000000000000     0 NOTYPE  WEAK   DEFAULT  UND _ITM_deregisterT[...]
    //      3: 0000000000000000     0 FUNC    WEAK   DEFAULT  UND [...]@GLIBC_2.2.5 (4)
    //      4: 0000000000000000     0 NOTYPE  WEAK   DEFAULT  UND _ITM_registerTMC[...]
    //      5: 0000000000002009     0 NOTYPE  GLOBAL DEFAULT   16 _end
    //      6: 0000000000002008     0 NOTYPE  GLOBAL DEFAULT   16 _edata
    //      7: 0000000000002008     0 NOTYPE  GLOBAL DEFAULT   16 __bss_start
    //      8: 0000000000000000     0 OBJECT  GLOBAL DEFAULT  ABS v1
    //      9: 00000000000006d8    15 FUNC    GLOBAL DEFAULT   12 hello_world@@v2
    //     10: 00000000000006c9    15 FUNC    GLOBAL DEFAULT   12 hello_world@v1
    //     11: 0000000000000000     0 OBJECT  GLOBAL DEFAULT  ABS v2

    // Undefined symbols are symbols that are required by the library.
    !symbol.is_undefined() &&
    // Symbols without a type are not interesting when linking.
    symbol.st_symtype() != STT_NOTYPE &&
    // Local symbols are an internal implementation detail.
    symbol.st_bind() != STB_LOCAL &&
    // Filter out the symbol version namespace entry.
    !(symbol.st_shndx == SHN_ABS && symbol.st_value == 0 && symbol.st_symtype() == STT_OBJECT)
}

/// Returns a list of all the exported symbols in the ELF file.
fn get_exported_symbols(path: &Path) -> Result<Vec<Symbol>> {
    let file = std::fs::File::open(path).with_context(|| format!("open {path:?}"))?;
    let mut elf = ElfStream::<AnyEndian, _>::open_stream(file)
        .with_context(|| format!("{path:?} is not a valid ELF"))?;

    let Some((dynamic_symbol_table, dynamic_symbol_string_table)) = elf
        .dynamic_symbol_table()
        .with_context(|| format!("Failed to parse dynamic symbol table from {path:?}"))?
    else {
        eprintln!("{path:?} doesn't have a dynamic symbol table");
        return Ok(vec![]);
    };

    // We can't have a `dynamic_symbol_table` and a `symbol_version_table` instantiated
    // at the same time because they both take a &mut, so we split up the computation.
    let exported_symbols = dynamic_symbol_table
        .into_iter()
        .enumerate()
        .filter(|(_i, symbol)| is_exported_symbol(symbol))
        .map(|(i, symbol)| {
            Ok((
                i,
                dynamic_symbol_string_table
                    .get(symbol.st_name.try_into().unwrap())
                    .with_context(|| {
                        format!("Failed to read symbol name at offset {}", symbol.st_name)
                    })?
                    .to_owned(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    let Some(symbol_version_table) = elf
        .symbol_version_table()
        .with_context(|| format!("Failed to parse symbol version table from {path:?}"))?
    else {
        eprintln!("{path:?} doesn't have a symbol version table");
        return Ok(exported_symbols
            .into_iter()
            .map(|(_i, name)| Symbol {
                name: name,
                version: None,
                hidden: false,
            })
            .collect());
    };

    let mut symbols = vec![];

    for (i, symbol_name) in exported_symbols {
        let Some(version_definition) = symbol_version_table
            .get_definition(i)
            .with_context(|| format!("Failed parsing definition table"))?
        else {
            eprintln!("Symbol {}:{} is not versioned?", i, symbol_name);
            symbols.push(Symbol {
                name: symbol_name,
                version: None,
                hidden: false,
            });
            continue;
        };

        for name in version_definition.names {
            let name = name.with_context(|| {
                format!("Failed while parsing symbol version for symbol {i}:{symbol_name}.")
            })?;

            symbols.push(Symbol {
                name: symbol_name.to_owned(),
                version: if version_definition.flags & VER_FLG_BASE != 0 {
                    // A base version means it's not versioned.
                    None
                } else {
                    Some(name.to_owned())
                },
                hidden: version_definition.hidden,
            });
        }
    }

    Ok(symbols)
}

/// Checks if the ELF file contains any versioned symbols.
pub fn has_versioned_symbols(path: &Path) -> Result<bool> {
    Ok(get_exported_symbols(path)?
        .into_iter()
        .any(|symbol| symbol.version.is_some()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::bail;
    use runfiles::Runfiles;
    use std::path::PathBuf;

    const BASE_DIR: &str = "cros/bazel/portage/bin/create_interface_layer";

    fn lookup_runfile(runfile_path: impl AsRef<Path>) -> Result<PathBuf> {
        let r = Runfiles::create()?;
        let full_path = runfiles::rlocation!(r, Path::new(BASE_DIR).join(runfile_path.as_ref()));
        if !full_path.try_exists()? {
            bail!("{full_path:?} does not exist");
        }

        Ok(full_path)
    }

    #[test]
    fn test_simple_lib() -> Result<()> {
        let lib_path = lookup_runfile("simple_lib.so")?;

        assert_eq!(
            &get_exported_symbols(&lib_path)?,
            &[Symbol {
                name: "hello_world".to_owned(),
                version: None,
                hidden: false
            },]
        );
        assert!(!has_versioned_symbols(&lib_path)?);
        Ok(())
    }

    #[test]
    fn test_simple_versioned_lib() -> Result<()> {
        let lib_path = lookup_runfile("simple_versioned_lib.so")?;

        assert_eq!(
            &get_exported_symbols(&lib_path)?,
            &[
                Symbol {
                    name: "hello_world".to_owned(),
                    version: Some("v2".to_owned()),
                    hidden: false,
                },
                Symbol {
                    name: "hello_world".to_owned(),
                    version: Some("v1".to_owned()),
                    hidden: true,
                },
            ]
        );
        assert!(has_versioned_symbols(&lib_path)?);
        Ok(())
    }
}

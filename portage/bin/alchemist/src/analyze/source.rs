// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{
    bash::expr::BashExpr, config::bundle::ConfigBundle, dependency::restrict::RestrictAtom,
};
use anyhow::{ensure, Context};
use std::{
    collections::{HashMap, HashSet},
    fs::{metadata, read_to_string},
    io::ErrorKind,
    iter::repeat,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, bail, Result};
use itertools::izip;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    bash::vars::BashValue,
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, parse_simplified_dependency, simplify},
        uri::{UriAtomDependency, UriDependency},
    },
    ebuild::PackageDetails,
};

use super::restrict::analyze_restricts;

/// Represents a chrome version number
/// i.e., 113.0.5623.0
pub type ChromeVersion = String;

/// Represents an origin of local source code.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PackageLocalSource {
    /// A pre-configured bazel target.
    /// We make this the first item in the enum, so that when the sources get
    /// sorted, this comes first, and thus results in being in the higher
    /// overlay fs layer.
    BazelTarget(String),
    /// ChromeOS source code at `/mnt/host/source/src`.
    Src(PathBuf),
    /// Chromite source code at `/mnt/host/source/chromite`.
    Chromite,
    /// Chrome source code.
    Chrome(ChromeVersion),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PackageRepoSource {
    pub name: String,
    pub project: String,
    pub tree_hash: String,
    pub project_path: PathBuf,
    pub subtree: Option<PathBuf>,
}
impl PackageRepoSource {
    pub fn full_path(&self) -> PathBuf {
        match &self.subtree {
            Some(subtree) => self.project_path.join(subtree),
            None => self.project_path.clone(),
        }
    }
}

/// Represents a source code archive to be fetched remotely to build a package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageDistSource {
    pub urls: Vec<Url>,
    pub filename: String,
    pub size: u64,
    pub hashes: HashMap<String, String>,
}

impl PackageDistSource {
    pub fn compute_integrity(&self) -> Result<String> {
        // We prefer SHA512 for integrity checking.
        for name in ["SHA512", "SHA256", "BLAKE2B"] {
            let hash_hex = match self.hashes.get(name) {
                Some(hash) => hash,
                None => continue,
            };
            let hash_bytes = hex::decode(hash_hex)?;
            let hash_base64 = base64::encode(hash_bytes);
            return Ok(format!("{}-{}", name.to_ascii_lowercase(), hash_base64));
        }
        bail!("No supported hash found in Manifest");
    }
}

/// Analyzed source information of a package. It is returned by
/// [`analyze_sources`].
pub struct PackageSources {
    pub local_sources: Vec<PackageLocalSource>,
    pub repo_sources: Vec<PackageRepoSource>,
    pub dist_sources: Vec<PackageDistSource>,
}

fn get_cros_workon_array_variable(
    details: &PackageDetails,
    name: &str,
    projects: usize,
) -> Result<Vec<String>> {
    let raw_values = match details.vars.hash_map().get(name) {
        None => {
            bail!("{} not defined", name);
        }
        Some(BashValue::Scalar(value)) => vec![value.clone()],
        Some(BashValue::IndexedArray(values)) => values.clone(),
        Some(other) => {
            bail!("Invalid {} value: {:?}", name, other);
        }
    };

    // If the number of elements is 1, repeat the same value for the number of
    // projects.
    let extended_values = if raw_values.len() == 1 {
        repeat(raw_values[0].clone()).take(projects).collect()
    } else {
        if raw_values.len() != projects {
            bail!(
                "Expected {} to have length of {}, got {}",
                name,
                projects,
                raw_values.len()
            );
        }
        raw_values
    };
    Ok(extended_values)
}

fn get_cros_workon_tree(details: &PackageDetails) -> Result<Vec<String>> {
    match details.vars.hash_map().get("CROS_WORKON_TREE") {
        None => {
            bail!("CROS_WORKON_TREE not defined");
        }
        Some(BashValue::Scalar(value)) => {
            if value.is_empty() {
                Ok(Vec::new())
            } else {
                Ok(Vec::from([value.clone()]))
            }
        }
        Some(BashValue::IndexedArray(values)) => Ok(values.clone()),
        Some(other) => {
            bail!("Invalid CROS_WORKON_TREE value: {:?}", other);
        }
    }
}

fn extract_cros_workon_sources(
    details: &PackageDetails,
    src_dir: &Path,
) -> Result<(Vec<PackageLocalSource>, Vec<PackageRepoSource>)> {
    let projects = match details.vars.hash_map().get("CROS_WORKON_PROJECT") {
        None => {
            // This is not a cros-workon package.
            return Ok((Vec::new(), Vec::new()));
        }
        Some(BashValue::Scalar(project)) => vec![project.clone()],
        Some(BashValue::IndexedArray(projects)) => projects.clone(),
        others => {
            bail!("Invalid CROS_WORKON_PROJECT value: {:?}", others)
        }
    };

    let local_names =
        get_cros_workon_array_variable(details, "CROS_WORKON_LOCALNAME", projects.len())?;
    let subtrees = get_cros_workon_array_variable(details, "CROS_WORKON_SUBTREE", projects.len())?;
    let optional_expressions =
        get_cros_workon_array_variable(details, "CROS_WORKON_OPTIONAL_CHECKOUT", projects.len())?;
    let trees = get_cros_workon_tree(details)?;

    let is_chromeos_base = details.package_name.starts_with("chromeos-base/");

    let mut source_paths = Vec::<PathBuf>::new();
    let mut repo_sources = Vec::<PackageRepoSource>::new();
    let mut seen_trees = HashSet::<&String>::new();

    let mut tree_index = 0;
    for (project, local_name, subtree, optional_expression) in
        izip!(&projects, local_names, &subtrees, optional_expressions)
    {
        // CROS_WORKON_LOCALNAME points to file paths relative to src/ if the
        // package is in the chromeos-base category; otherwise they're relative
        // to src/third_party/.
        let local_path = if local_name == "chromiumos-assets" {
            // HACK: chromiumos-assets ebuild is incorrect.
            // TODO: Fix the ebuild and remove this hack.
            "platform/chromiumos-assets".to_owned()
        } else if is_chromeos_base {
            local_name
        } else if let Some(clean_path) = local_name.strip_prefix("../") {
            clean_path.to_owned()
        } else {
            format!("third_party/{}", local_name)
        };

        let required = if optional_expression.is_empty() {
            true
        } else {
            BashExpr::from_str(&optional_expression)
                .with_context(|| format!("Expression '{}'", optional_expression))?
                .eval(&details.use_map)?
        };

        let local_subtrees = if subtree.is_empty() {
            Vec::from([""])
        } else {
            subtree.split_ascii_whitespace().collect_vec()
        };

        if trees.is_empty() {
            // 9999 ebuilds
            if !required {
                // Skip the whole project
                continue;
            }
            for subtree in local_subtrees {
                let subtree = subtree.trim_start_matches('/');

                if subtree.is_empty() {
                    source_paths.push(PathBuf::from(&local_path))
                } else {
                    source_paths.push(Path::new(&local_path).join(subtree))
                }
            }
        } else {
            // Pinned ebuilds
            for subtree in local_subtrees {
                if tree_index >= trees.len() {
                    bail!("invalid number of entries in CROS_WORKON_TREE {:?}", &trees);
                }
                let tree_hash = &trees[tree_index];
                tree_index += 1;

                // Even if the project isn't required, we still need to increment
                // the tree_index.
                if !required {
                    continue;
                }
                if !seen_trees.insert(tree_hash) {
                    // There are two possible reasons a package could have duplicate hashes:
                    // 1) The package incorrectly declares a duplicate entry in SUBTREE.
                    // 2) Two subtrees end up being identical.
                    //
                    // Fortunately trees don't have an order requirement so we can just skip
                    // adding the duplicate.
                    continue;
                }

                repo_sources.push(PackageRepoSource {
                    name: format!("tree-{}-{}", project.replace('/', "-"), tree_hash),
                    project: project.to_string(),
                    tree_hash: tree_hash.to_string(),
                    project_path: PathBuf::from(&local_path),
                    subtree: if subtree.is_empty() {
                        None
                    } else {
                        Some(subtree.into())
                    },
                });
            }
        }
    }

    // Handle regular/missing files.
    let source_dirs: Vec<PathBuf> = source_paths
        .into_iter()
        .flat_map(|path| {
            let full_path = src_dir.join(&path);

            let meta = match metadata(&full_path) {
                Err(err) if err.kind() == ErrorKind::NotFound => {
                    // CROS_WORKON_SUBTREE may contain missing files.
                    // TODO(b/281793145): Remove this logic once cros-workon.eclass
                    // starts to reject non-existent files.
                    return None;
                }
                Err(err) => {
                    return Some(
                        Err(err).context(format!("failed to stat {}", full_path.display())),
                    );
                }
                Ok(meta) => meta,
            };

            if meta.is_dir() {
                Some(Ok(path))
            } else {
                // If the file is a regular file, use its parent directory.
                // TODO: Improve this to include the file only.
                Some(Ok(path.parent().unwrap().to_owned()))
            }
        })
        .collect::<Result<_>>()?;

    let mut sources = source_dirs
        .into_iter()
        .map(PackageLocalSource::Src)
        .collect_vec();

    // Kernel packages need extra eclasses.
    // TODO: Remove this hack.
    if projects
        .iter()
        .any(|p| p == "chromiumos/third_party/kernel")
    {
        sources.push(PackageLocalSource::Src(
            "third_party/chromiumos-overlay/eclass/cros-kernel".into(),
        ));
    }

    Ok((sources, repo_sources))
}

fn apply_local_sources_workarounds(
    config: &ConfigBundle,
    details: &PackageDetails,
    local_sources: &mut Vec<PackageLocalSource>,
) -> Result<()> {
    // We can't support the 9999 ebuild flow for the chrome ebuilds because
    // 1) We don't know where the chrome source is checked out, 2) We need to
    // run all the repo hooks to generate a self contained tarball.
    if details.inherited.contains("chromium-source")
        && details
            .version
            .main()
            .first()
            .map_or(false, |main| main != "9999")
    {
        let version = details.version.main().join(".");
        // TODO: We need USE flags to add src-internal.
        local_sources.push(PackageLocalSource::Chrome(version));
    }

    // The meson eclass calls `meson_test.py` in platform2/common-mk.
    if details.inherited.contains("meson") {
        // On ARM, we replace meson_test.py with a fake which does nothing because it doesn't work
        // yet.
        // TODO(b/272275535): Delete this hack once platform2_test.py works on ARM.
        let arch = config.env().get("ARCH").unwrap();
        if arch == "arm64" {
            local_sources.push(PackageLocalSource::BazelTarget(
                "@//bazel/portage/sdk:meson_test_disable_hack".to_string(),
            ));
        } else {
            let common_mk = PackageLocalSource::Src("platform2/common-mk".into());
            if !local_sources.contains(&common_mk) {
                local_sources.push(common_mk)
            }
        }
    }

    // Running install hooks requires src/scripts/hooks and chromite.
    local_sources.push(PackageLocalSource::Src("scripts/hooks".into()));
    local_sources.push(PackageLocalSource::Chromite);

    // The platform eclass calls `platform2_test.py`.
    // The meson eclass calls `meson_test.py` which calls `platform2_test.py`.
    // TODO(b/272275535): Delete this once platform2_test.py works
    if details.inherited.contains("platform") || details.inherited.contains("meson") {
        local_sources.push(PackageLocalSource::BazelTarget(
            "@//bazel/portage/sdk:platform2_test_hack".to_string(),
        ));
    }

    Ok(())
}

fn parse_uri_dependencies(deps: UriDependency, use_map: &UseMap) -> Result<Vec<UriAtomDependency>> {
    let deps = elide_use_conditions(deps, use_map).unwrap_or_default();
    let deps = simplify(deps);
    parse_simplified_dependency(deps)
}

struct DistEntry {
    pub filename: String,
    pub size: u64,
    pub hashes: HashMap<String, String>,
}

struct PackageManifest {
    pub dists: Vec<DistEntry>,
}

fn load_package_manifest(dir: &Path) -> Result<PackageManifest> {
    let full_path = &dir.join("Manifest");
    let content = read_to_string(full_path).with_context(|| full_path.display().to_string())?;
    let dists = content
        .split('\n')
        .map(|line| {
            let mut columns = line.split_ascii_whitespace().map(|s| s.to_owned());

            let dist = columns.next();
            match dist {
                Some(s) if s == "DIST" => {}
                _ => return Ok(None), // ignore other lines
            }

            let (filename, size_str) = columns
                .next_tuple()
                .ok_or_else(|| anyhow!("Corrupted Manifest line: {}", line))?;
            let size = size_str.parse()?;
            let hashes = HashMap::<String, String>::from_iter(columns.tuples());
            Ok(Some(DistEntry {
                filename,
                size,
                hashes,
            }))
        })
        .flatten_ok()
        .collect::<Result<Vec<_>>>()?;
    Ok(PackageManifest { dists })
}

// These are the only public gs buckets an ebuild should be accessing.
// See https://source.chromium.org/chromium/chromiumos/docs/+/main:archive_mirrors.md
static PUBLIC_GS_BUCKETS: &[&str] = &[
    "chromeos-mirror",
    "chromeos-localmirror",
    // These have not been listed in the doc above, but are considered valid.
    // See b/271483241.
    "chromium-nodejs",
    "chromeos-prebuilt",
    "termina-component-testing",
];

// For the public mirrors, lets prefer using HTTPS to download the files.
// TODO(b/271846096): Delete this if we choose.
fn convert_public_gs_buckets_to_https(url: Url) -> Result<Url> {
    let host = url.host_str().expect("URL to have a host");

    if url.scheme() != "gs" || !PUBLIC_GS_BUCKETS.contains(&host) {
        return Ok(url);
    }
    Ok(Url::parse(
        format!("https://storage.googleapis.com/{}{}", host, url.path()).as_ref(),
    )?)
}

fn extract_remote_sources(
    config: &ConfigBundle,
    details: &PackageDetails,
) -> Result<Vec<PackageDistSource>> {
    let restricts = analyze_restricts(details)?;

    // TODO: We should read the FEATURES field from the portage config and check
    // for `mirror` and `force-mirror`. For our purposes we always want
    // force-mirror so let's just hard code it for now.
    let force_mirror = true;

    let use_mirror = force_mirror && !restricts.contains(&RestrictAtom::Mirror);

    let mirrors = if use_mirror {
        let mirrors = config
            .env()
            .get("GENTOO_MIRRORS")
            .map_or("", |s| s.as_str());
        let mirrors = mirrors.split_ascii_whitespace().collect_vec();

        ensure!(
            !mirrors.is_empty(),
            "Force mirror is enabled, but no mirrors were found"
        );

        Some(mirrors)
    } else {
        None
    };

    // Collect URIs from SRC_URI.
    let src_uri = details.vars.get_scalar_or_default("SRC_URI")?;
    let source_deps = src_uri.parse::<UriDependency>()?;
    let source_atoms = parse_uri_dependencies(source_deps, &details.use_map)?;

    // Construct a map from file names to URIs.
    let mut source_map = HashMap::<String, Vec<Url>>::new();
    for source_atom in source_atoms {
        let (url, filename) = match source_atom {
            UriAtomDependency::Uri(mut url, opt_filename) => {
                let filename = if let Some(filename) = opt_filename {
                    filename
                } else if let Some(segments) = url.path_segments() {
                    segments.last().unwrap().to_owned()
                } else {
                    bail!("Invalid source URI: {}", &url);
                };

                // TODO: Fix ebuilds with bad URLS. For now let's fix the URL
                // for them.
                url.set_path(&url.path().replace("//", "/"));

                (url, filename)
            }
            UriAtomDependency::Filename(filename) => {
                bail!("Found non-URI source: {}", &filename);
            }
        };
        source_map.entry(filename).or_default().push(url);
    }

    if source_map.is_empty() {
        return Ok(Vec::new());
    }

    let manifest = load_package_manifest(details.ebuild_path.parent().unwrap())?;

    let mut dist_map: HashMap<String, DistEntry> = manifest
        .dists
        .into_iter()
        .map(|dist| (dist.filename.clone(), dist))
        .collect();

    let mut sources = source_map
        .into_iter()
        .map(|(filename, urls)| {
            let dist = dist_map
                .remove(&filename)
                .ok_or_else(|| anyhow!("{} not found in Manifest", &filename))?;

            let urls = match &mirrors {
                Some(mirrors) => mirrors
                    .iter()
                    .map(|mirror| Url::parse(format!("{}/distfiles/{}", mirror, filename).as_ref()))
                    .collect::<::core::result::Result<_, _>>()?,
                None => urls
                    .into_iter()
                    .map(convert_public_gs_buckets_to_https)
                    .collect::<Result<Vec<_>>>()?,
            };

            // TODO: This should probably go into generate_repo, but failing
            // there is fatal. If we fail here then we at least get a nice error
            // message when using --verbose.
            for url in &urls {
                ensure!(
                    // Some ebuilds specify http://
                    url.scheme() == "https"
                        || url.scheme() == "http"
                        || url.scheme() == "cipd"
                        || url.scheme() == "gs",
                    "Only http/https/cipd/gs URLs are supported, got {}",
                    url
                );
            }

            Ok(PackageDistSource {
                urls,
                filename,
                size: dist.size,
                hashes: dist.hashes,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    sources.sort_unstable_by(|a, b| a.filename.cmp(&b.filename));

    Ok(sources)
}

/// Analyzes ebuild variables and returns [`PackageSources`] summarizing its
/// source information.
pub fn analyze_sources(
    config: &ConfigBundle,
    details: &PackageDetails,
    src_dir: &Path,
) -> Result<PackageSources> {
    let (mut local_sources, repo_sources) = extract_cros_workon_sources(details, src_dir)?;

    // We sort before applying the workarounds because we want the
    // meson_test_disable_hack layer to be last so it overrides the common-mk
    // layer.
    local_sources.sort();
    local_sources.dedup();

    apply_local_sources_workarounds(config, details, &mut local_sources)?;

    Ok(PackageSources {
        local_sources,
        repo_sources,
        dist_sources: extract_remote_sources(config, details)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bash::vars::BashVars;
    use crate::data::{Slot, Vars};
    use crate::testutils::write_files;
    use std::collections::HashSet;

    use tempfile::TempDir;
    use version::Version;

    const MIRRORS: &str = "https://mirror/a https://mirror/b";

    fn new_non_cros_workon_package(use_map: UseMap) -> Result<(PackageDetails, TempDir)> {
        let tmp = TempDir::new()?;

        write_files(
            tmp.path(),
            [(
                "Manifest",
                r#"
                DIST foo-0.1.0.tar.gz 12345 SHA256 01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b
                DIST foo-extra.tar.gz 56789 SHA256 a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447
                "#,
            )],
        )?;

        let package = PackageDetails {
            repo_name: "baz".to_owned(),
            package_name: "sys-libs/foo".to_owned(),
            version: Version::try_new("0.1.0").unwrap(),
            vars: BashVars::new(HashMap::from([
                ("SRC_URI".to_owned(),
                    BashValue::Scalar("https://example/f00-0.1.0.tar.gz -> foo-0.1.0.tar.gz extra? ( gs://chromeos-localmirror/foo-extra.tar.gz )".to_owned())),
                ("RESTRICT".to_owned(),
                    BashValue::Scalar("extra? ( mirror )".to_owned())),
                ])),
            slot: Slot::new("0"),
            use_map,
            accepted: true,
            stable: true,
            masked: false,
            ebuild_path: tmp.path().join("foo-0.1.0.ebuild"),
            inherited: HashSet::new(),
            direct_build_target: None,
        };

        Ok((package, tmp))
    }

    #[test]
    fn non_cros_workon_package() -> Result<()> {
        let (package, _tmpdir) = new_non_cros_workon_package(UseMap::new())?;
        let (local_sources, repo_sources) =
            extract_cros_workon_sources(&package, Path::new("/src"))?;

        assert_eq!(local_sources, []);
        assert_eq!(repo_sources, []);

        Ok(())
    }

    #[test]
    fn src_uri_mirror_no_extra() -> Result<()> {
        let config = ConfigBundle::new(
            Vars::from([("GENTOO_MIRRORS".to_owned(), MIRRORS.to_owned())]),
            vec![],
        );

        let (package, _tmpdir) = new_non_cros_workon_package(UseMap::new())?;
        let dist_sources = extract_remote_sources(&config, &package)?;

        assert_eq!(
            dist_sources,
            [PackageDistSource {
                urls: vec![
                    Url::parse("https://mirror/a/distfiles/foo-0.1.0.tar.gz")?,
                    Url::parse("https://mirror/b/distfiles/foo-0.1.0.tar.gz")?,
                ],
                filename: "foo-0.1.0.tar.gz".to_string(),
                size: 12345,
                hashes: HashMap::from([(
                    "SHA256".to_string(),
                    "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b".to_string()
                )]),
            }]
        );

        Ok(())
    }

    #[test]
    fn src_uri_mirror_with_extra() -> Result<()> {
        let config = ConfigBundle::new(
            Vars::from([("GENTOO_MIRRORS".to_owned(), MIRRORS.to_owned())]),
            vec![],
        );

        let (package, _tmpdir) =
            new_non_cros_workon_package(UseMap::from([("extra".to_string(), true)]))?;
        let dist_sources = extract_remote_sources(&config, &package)?;

        assert_eq!(
            dist_sources,
            [
                PackageDistSource {
                    urls: vec![Url::parse("https://example/f00-0.1.0.tar.gz")?,],
                    filename: "foo-0.1.0.tar.gz".to_string(),
                    size: 12345,
                    hashes: HashMap::from([(
                        "SHA256".to_string(),
                        "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b"
                            .to_string()
                    )]),
                },
                PackageDistSource {
                    urls: vec![Url::parse(
                        "https://storage.googleapis.com/chromeos-localmirror/foo-extra.tar.gz"
                    )?,],
                    filename: "foo-extra.tar.gz".to_string(),
                    size: 56789,
                    hashes: HashMap::from([(
                        "SHA256".to_string(),
                        "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447"
                            .to_string()
                    )]),
                }
            ]
        );

        Ok(())
    }

    #[test]
    fn cros_workon_pinned_package_with_subtree() -> Result<()> {
        let package = PackageDetails {
            repo_name: "baz".to_owned(),
            package_name: "sys-boot/libpayload".to_owned(),
            version: Version::try_new("0.1.0")?,
            vars: BashVars::new(HashMap::from([
                (
                    "CROS_WORKON_PROJECT".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "chromiumos/third_party/coreboot".to_owned(),
                        "chromiumos/platform/vboot_reference".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_LOCALNAME".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "coreboot".to_owned(),
                        "../platform/vboot_reference".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_SUBTREE".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "payloads/libpayload src/commonlib util/kconfig util/xcompile".to_owned(),
                        "Makefile firmware".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_COMMIT".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "e71dd376a369e2351265e79e19e926594f92e604".to_owned(),
                        "49820c727819ca566c65efa0525a8022f07cc27e".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_TREE".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "6f11773570dfaaade362374b0d0392c28cf17206".to_owned(),
                        "5e822365b04b4690729ca6ec32935a177db97ed2".to_owned(),
                        "514603540da793957fa87fa22df81b288fb39d0f".to_owned(),
                        "b2307ed1e70bf1a5718afaa81217ec9504854005".to_owned(),
                        "bc55f0377f73029f50c4c74d5936e4d7bde877c6".to_owned(),
                        "e70ebd7c76b9f9ad44b59e3002a5c57be5b9dc12".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_OPTIONAL_CHECKOUT".to_owned(),
                    BashValue::IndexedArray(Vec::from(["".to_owned(), "".to_owned()])),
                ),
            ])),
            slot: Slot::new("0"),
            use_map: HashMap::new(),
            accepted: true,
            stable: true,
            masked: false,
            ebuild_path: PathBuf::from("/dev/null"),
            inherited: HashSet::new(),
            direct_build_target: None,
        };
        let (local_sources, repo_sources) =
            extract_cros_workon_sources(&package, Path::new("/src"))?;

        assert_eq!(local_sources, []);
        assert_eq!(
            repo_sources,
            [
                PackageRepoSource {
                    name: "tree-chromiumos-third_party-coreboot-6f11773570dfaaade362374b0d0392c28cf17206".into(),
                    project: "chromiumos/third_party/coreboot".into(),
                    tree_hash: "6f11773570dfaaade362374b0d0392c28cf17206".into(),
                    project_path: "third_party/coreboot".into(),
                    subtree: Some("payloads/libpayload".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-third_party-coreboot-5e822365b04b4690729ca6ec32935a177db97ed2".into(),
                    project: "chromiumos/third_party/coreboot".into(),
                    tree_hash: "5e822365b04b4690729ca6ec32935a177db97ed2".into(),
                    project_path: "third_party/coreboot".into(),
                    subtree: Some("src/commonlib".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-third_party-coreboot-514603540da793957fa87fa22df81b288fb39d0f".into(),
                    project: "chromiumos/third_party/coreboot".into(),
                    tree_hash: "514603540da793957fa87fa22df81b288fb39d0f".into(),
                    project_path: "third_party/coreboot".into(),
                    subtree: Some("util/kconfig".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-third_party-coreboot-b2307ed1e70bf1a5718afaa81217ec9504854005".into(),
                    project: "chromiumos/third_party/coreboot".into(),
                    tree_hash: "b2307ed1e70bf1a5718afaa81217ec9504854005".into(),
                    project_path: "third_party/coreboot".into(),
                    subtree: Some("util/xcompile".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-platform-vboot_reference-bc55f0377f73029f50c4c74d5936e4d7bde877c6".into(),
                    project: "chromiumos/platform/vboot_reference".into(),
                    tree_hash: "bc55f0377f73029f50c4c74d5936e4d7bde877c6".into(),
                    project_path: "platform/vboot_reference".into(),
                    subtree: Some("Makefile".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-platform-vboot_reference-e70ebd7c76b9f9ad44b59e3002a5c57be5b9dc12".into(),
                    project: "chromiumos/platform/vboot_reference".into(),
                    tree_hash: "e70ebd7c76b9f9ad44b59e3002a5c57be5b9dc12".into(),
                    project_path: "platform/vboot_reference".into(),
                    subtree: Some("firmware".into()),
                }
            ]
        );

        Ok(())
    }

    #[test]
    fn cros_workon_pinned_package_without_subtree() -> Result<()> {
        let package = PackageDetails {
            repo_name: "baz".to_owned(),
            package_name: "sys-boot/depthcharge".to_owned(),
            version: Version::try_new("0.1.0")?,
            vars: BashVars::new(HashMap::from([
                (
                    "CROS_WORKON_PROJECT".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "chromiumos/platform/depthcharge".to_owned(),
                        "chromiumos/platform/vboot_reference".to_owned(),
                        "chromiumos/third_party/coreboot".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_LOCALNAME".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "../platform/depthcharge".to_owned(),
                        "../platform/vboot_reference".to_owned(),
                        "../third_party/coreboot".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_COMMIT".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "7e1e4037a9e46a9cbf502b2b20cdc9db1a84cf94".to_owned(),
                        "52f28a4b68aa018fff3cc575610bc9c1c04a030f".to_owned(),
                        "d5929971f3efe2e8a398c385309ca4aad110dc02".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_TREE".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "63534c063f7717bd89631830e076229c41829c17".to_owned(),
                        "b7ba676717ca1fa2a26b1f3107afdce3be979a78".to_owned(),
                        "5478a5900ed6376f77b84efb27677c105fc253d6".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_SUBTREE".to_owned(),
                    BashValue::Scalar("".to_owned()),
                ),
                (
                    "CROS_WORKON_OPTIONAL_CHECKOUT".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "".to_owned(),
                        "".to_owned(),
                        "".to_owned(),
                    ])),
                ),
            ])),
            slot: Slot::new("0"),
            use_map: HashMap::new(),
            accepted: true,
            stable: true,
            masked: false,
            ebuild_path: PathBuf::from("/dev/null"),
            inherited: HashSet::new(),
            direct_build_target: None,
        };
        let (local_sources, repo_sources) =
            extract_cros_workon_sources(&package, Path::new("/src"))?;

        assert_eq!(local_sources, []);
        assert_eq!(
            repo_sources,
            [
                PackageRepoSource {
                    name: "tree-chromiumos-platform-depthcharge-63534c063f7717bd89631830e076229c41829c17".into(),
                    project: "chromiumos/platform/depthcharge".into(),
                    tree_hash: "63534c063f7717bd89631830e076229c41829c17".into(),
                    project_path: "platform/depthcharge".into(),
                    subtree: None,
                },
                PackageRepoSource {
                    name: "tree-chromiumos-platform-vboot_reference-b7ba676717ca1fa2a26b1f3107afdce3be979a78".into(),
                    project: "chromiumos/platform/vboot_reference".into(),
                    tree_hash: "b7ba676717ca1fa2a26b1f3107afdce3be979a78".into(),
                    project_path: "platform/vboot_reference".into(),
                    subtree: None,
                },
                PackageRepoSource {
                    name: "tree-chromiumos-third_party-coreboot-5478a5900ed6376f77b84efb27677c105fc253d6".into(),
                    project: "chromiumos/third_party/coreboot".into(),
                    tree_hash: "5478a5900ed6376f77b84efb27677c105fc253d6".into(),
                    project_path: "third_party/coreboot".into(),
                    subtree: None,
                },

            ]
        );

        Ok(())
    }

    fn create_optional_subtree_package(use_map: UseMap) -> PackageDetails {
        PackageDetails {
            repo_name: "baz".to_owned(),
            package_name: "sys-boot/libpayload".to_owned(),
            version: Version::try_new("0.1.0").unwrap(),
            vars: BashVars::new(HashMap::from([
                (
                    "CROS_WORKON_PROJECT".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "chromiumos/third_party/coreboot".to_owned(),
                        "chromiumos/platform/vboot_reference".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_LOCALNAME".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "coreboot".to_owned(),
                        "../platform/vboot_reference".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_SUBTREE".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "payloads/libpayload src/commonlib util/kconfig util/xcompile".to_owned(),
                        "Makefile firmware".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_COMMIT".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "e71dd376a369e2351265e79e19e926594f92e604".to_owned(),
                        "49820c727819ca566c65efa0525a8022f07cc27e".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_TREE".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "6f11773570dfaaade362374b0d0392c28cf17206".to_owned(),
                        "5e822365b04b4690729ca6ec32935a177db97ed2".to_owned(),
                        "514603540da793957fa87fa22df81b288fb39d0f".to_owned(),
                        "b2307ed1e70bf1a5718afaa81217ec9504854005".to_owned(),
                        "bc55f0377f73029f50c4c74d5936e4d7bde877c6".to_owned(),
                        "e70ebd7c76b9f9ad44b59e3002a5c57be5b9dc12".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_OPTIONAL_CHECKOUT".to_owned(),
                    BashValue::IndexedArray(Vec::from(["use coreboot".to_owned(), "".to_owned()])),
                ),
            ])),
            slot: Slot::new("0"),
            use_map,
            accepted: true,
            stable: true,
            masked: false,
            ebuild_path: PathBuf::from("/dev/null"),
            inherited: HashSet::new(),
            direct_build_target: None,
        }
    }

    #[test]
    fn cros_workon_pinned_package_with_subtree_optional_checkout_true() -> Result<()> {
        let package =
            create_optional_subtree_package(UseMap::from([("coreboot".to_owned(), true)]));

        let (local_sources, repo_sources) =
            extract_cros_workon_sources(&package, Path::new("/src"))?;

        assert_eq!(local_sources, []);
        assert_eq!(
            repo_sources,
            [
                PackageRepoSource {
                    name: "tree-chromiumos-third_party-coreboot-6f11773570dfaaade362374b0d0392c28cf17206".into(),
                    project: "chromiumos/third_party/coreboot".into(),
                    tree_hash: "6f11773570dfaaade362374b0d0392c28cf17206".into(),
                    project_path: "third_party/coreboot".into(),
                    subtree: Some("payloads/libpayload".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-third_party-coreboot-5e822365b04b4690729ca6ec32935a177db97ed2".into(),
                    project: "chromiumos/third_party/coreboot".into(),
                    tree_hash: "5e822365b04b4690729ca6ec32935a177db97ed2".into(),
                    project_path: "third_party/coreboot".into(),
                    subtree: Some("src/commonlib".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-third_party-coreboot-514603540da793957fa87fa22df81b288fb39d0f".into(),
                    project: "chromiumos/third_party/coreboot".into(),
                    tree_hash: "514603540da793957fa87fa22df81b288fb39d0f".into(),
                    project_path: "third_party/coreboot".into(),
                    subtree: Some("util/kconfig".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-third_party-coreboot-b2307ed1e70bf1a5718afaa81217ec9504854005".into(),
                    project: "chromiumos/third_party/coreboot".into(),
                    tree_hash: "b2307ed1e70bf1a5718afaa81217ec9504854005".into(),
                    project_path: "third_party/coreboot".into(),
                    subtree: Some("util/xcompile".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-platform-vboot_reference-bc55f0377f73029f50c4c74d5936e4d7bde877c6".into(),
                    project: "chromiumos/platform/vboot_reference".into(),
                    tree_hash: "bc55f0377f73029f50c4c74d5936e4d7bde877c6".into(),
                    project_path: "platform/vboot_reference".into(),
                    subtree: Some("Makefile".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-platform-vboot_reference-e70ebd7c76b9f9ad44b59e3002a5c57be5b9dc12".into(),
                    project: "chromiumos/platform/vboot_reference".into(),
                    tree_hash: "e70ebd7c76b9f9ad44b59e3002a5c57be5b9dc12".into(),
                    project_path: "platform/vboot_reference".into(),
                    subtree: Some("firmware".into()),
                }
            ]
        );

        Ok(())
    }

    #[test]
    fn cros_workon_pinned_package_with_subtree_optional_checkout_false() -> Result<()> {
        let package =
            create_optional_subtree_package(UseMap::from([("coreboot".to_owned(), false)]));

        let (local_sources, repo_sources) =
            extract_cros_workon_sources(&package, Path::new("/src"))?;

        assert_eq!(local_sources, []);
        assert_eq!(
            repo_sources,
            [
                PackageRepoSource {
                    name: "tree-chromiumos-platform-vboot_reference-bc55f0377f73029f50c4c74d5936e4d7bde877c6".into(),
                    project: "chromiumos/platform/vboot_reference".into(),
                    tree_hash: "bc55f0377f73029f50c4c74d5936e4d7bde877c6".into(),
                    project_path: "platform/vboot_reference".into(),
                    subtree: Some("Makefile".into()),
                },
                PackageRepoSource {
                    name: "tree-chromiumos-platform-vboot_reference-e70ebd7c76b9f9ad44b59e3002a5c57be5b9dc12".into(),
                    project: "chromiumos/platform/vboot_reference".into(),
                    tree_hash: "e70ebd7c76b9f9ad44b59e3002a5c57be5b9dc12".into(),
                    project_path: "platform/vboot_reference".into(),
                    subtree: Some("firmware".into()),
                }
            ]
        );

        Ok(())
    }

    // TODO: We need to construct a real src tree for this to work.
    #[ignore]
    #[test]
    fn cros_workon_9999_package() -> Result<()> {
        let package = PackageDetails {
            repo_name: "baz".to_owned(),
            package_name: "sys-boot/depthcharge".to_owned(),
            version: Version::try_new("9999")?,
            vars: BashVars::new(HashMap::from([
                (
                    "CROS_WORKON_PROJECT".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "chromiumos/platform/depthcharge".to_owned(),
                        "chromiumos/platform/vboot_reference".to_owned(),
                        "chromiumos/third_party/coreboot".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_LOCALNAME".to_owned(),
                    BashValue::IndexedArray(Vec::from([
                        "../platform/depthcharge".to_owned(),
                        "../platform/vboot_reference".to_owned(),
                        "../third_party/coreboot".to_owned(),
                    ])),
                ),
                (
                    "CROS_WORKON_COMMIT".to_owned(),
                    BashValue::Scalar("".to_owned()),
                ),
                (
                    "CROS_WORKON_TREE".to_owned(),
                    BashValue::Scalar("".to_owned()),
                ),
                (
                    "CROS_WORKON_SUBTREE".to_owned(),
                    BashValue::Scalar("".to_owned()),
                ),
            ])),
            slot: Slot::new("0"),
            use_map: HashMap::new(),
            accepted: true,
            stable: true,
            masked: false,
            ebuild_path: PathBuf::from("/dev/null"),
            inherited: HashSet::new(),
            direct_build_target: None,
        };
        let (local_sources, repo_sources) =
            extract_cros_workon_sources(&package, Path::new("/src"))?;

        assert_eq!(repo_sources, []);
        assert_eq!(
            local_sources,
            [
                PackageLocalSource::Src("platform/depthcharge".into()),
                PackageLocalSource::Src("platform/vboot_reference".into()),
                PackageLocalSource::Src("third_party/coreboot".into()),
            ]
        );

        Ok(())
    }
}
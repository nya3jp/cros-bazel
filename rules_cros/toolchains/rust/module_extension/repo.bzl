"""cros_rust_repository is a repository rule for importing the Rust toolchain from the CrOS SDK."""

load("//rules_cros/toolchains:platforms.bzl", "all_toolchain_descs", "desc_to_triple")

def _execute_bash(repo_ctx, cmd):
    return repo_ctx.execute(["/bin/bash", "-c", cmd]).stdout.strip("\n")

def _get_rustlib_filenames(repo_ctx, target_triple):
    filenames = _execute_bash(
        repo_ctx,
        "find /usr/lib64/rustlib/{}/lib -type f -printf '%P\\n'".format(target_triple),
    )
    if filenames:
        return filenames.split("\n")
    else:
        return None

_RUSTC_TOOLS = ["rustc", "cargo", "clippy-driver", "rustdoc", "rustfmt"]

def rust_repo(repo_ctx):
    # Symlink necessary rustc binaryes and libraries into this repository so
    # that they can be mapped to Bazel labels and tracked as proper dependencies
    # via filegroups.

    # The repository directory structure looks like:
    # rust/
    #   BUILD
    #   bin/
    #       BUILD
    #       {platform triple}/
    #           BUILD
    #           rustc, rustfmt, etc.
    #   lib/
    #       BUILD
    #       {platform triple}/
    #           BUILD
    #           lib/
    #               *.rlib
    #               *.a

    repo_ctx.file("rust/BUILD")

    # Symlink in necessary binaries mentioned in _RUSTC_TOOLS
    repo_ctx.file("rust/bin/BUILD")
    repo_ctx.file(
        "rust/bin/x86_64-pc-linux-gnu/BUILD",
        content = "exports_files({})\n".format(repr(_RUSTC_TOOLS)),
    )
    for tool in _RUSTC_TOOLS:
        repo_ctx.symlink(
            "/usr/bin/{}".format(tool),
            "rust/bin/x86_64-pc-linux-gnu/{}".format(tool),
        )

    repo_ctx.file("rust/lib/BUILD")

    # For each target triple, symlink in every file we find under the
    # appropriate rustlib directory.
    for desc in all_toolchain_descs:
        triple = desc_to_triple(desc)
        repo_ctx.symlink(
            repo_ctx.attr._rust_rustlib_build,
            "rust/lib/{triple}/BUILD".format(triple = triple),
        )

        lib_files = _get_rustlib_filenames(repo_ctx, triple)
        for rustc_file in lib_files:
            repo_ctx.symlink(
                "/usr/lib64/rustlib/{triple}/lib/{file}".format(triple = triple, file = rustc_file),
                "rust/lib/{triple}/lib/{file}".format(triple = triple, file = rustc_file),
            )
    repo_ctx.symlink(repo_ctx.attr._rust_toolchains_build, "rust/toolchains/BUILD.bazel")

RUST_ATTRS = dict(
    _rust_toolchains_build = attr.label(allow_single_file=True, default=Label("//rules_cros/toolchains/rust/module_extension:BUILD.toolchain.bazel")),
    _rust_rustlib_build = attr.label(allow_single_file=True, default=Label("//rules_cros/toolchains/rust/module_extension:BUILD.rustlib.bazel")),
)
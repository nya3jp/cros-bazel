load("//bazel/crates:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")

def _rust_crate(name, rule, deps = [], **kwargs):
    rule(
        name = name,
        srcs = native.glob(["src/**/*.rs"]),
        aliases = aliases(),
        proc_macro_deps = all_crate_deps(proc_macro = True),
        deps = deps + all_crate_deps(normal = True),
        **kwargs
    )

    native.exports_files(["Cargo.toml"])

def rust_library_crate(name, deps = [], **kwargs):
    _rust_crate(name, rule = rust_library, deps = deps, **kwargs)

def rust_binary_crate(name, **kwargs):
    _rust_crate(name, rule = rust_binary, **kwargs)

def rust_crate_test(name, crate, size = "small", deps = [], **kwargs):
    rust_test(
        name = name,
        crate = crate,
        aliases = aliases(
            normal_dev = True,
            proc_macro_dev = True,
        ),
        deps = all_crate_deps(
            normal_dev = True,
        ) + deps,
        proc_macro_deps = all_crate_deps(
            proc_macro_dev = True,
        ),
        size = size,
        **kwargs
    )

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")
load("@rules_rust//bindgen:repositories.bzl", "rust_bindgen_dependencies", "rust_bindgen_register_toolchains")
load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")
load("@rules_rust//tools/rust_analyzer:deps.bzl", "rust_analyzer_dependencies")
load("//bazel/crates:crates.bzl", "crate_repositories")

def rust_toolchains():
    rules_rust_dependencies()

    rust_register_toolchains(
        edition = "2021",
    )

    crate_universe_dependencies(bootstrap = True)
    rust_analyzer_dependencies()
    rust_bindgen_dependencies()
    rust_bindgen_register_toolchains()
    crate_repositories()

load("@rules_rust_non_bzlmod//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

def rust_toolchains():
    rules_rust_dependencies()

    rust_register_toolchains(
        edition = "2021",
    )

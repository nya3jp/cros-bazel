load("//rules_cros/toolchains/nasm:repositories.bzl", "nasm_repositories")
load("//rules_cros/toolchains/perl:repositories.bzl", "perl_repositories")
load("//rules_cros/toolchains/rust:repositories.bzl", "rust_repositories")

def toolchain_repositories():
    nasm_repositories()
    perl_repositories()
    rust_repositories()

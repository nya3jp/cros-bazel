load("//bazel/toolchains/nasm:repositories.bzl", "nasm_repositories")
load("//bazel/toolchains/python:repositories.bzl", "python_repositories")
load("//bazel/toolchains/perl:repositories.bzl", "perl_repositories")
load("//bazel/toolchains/rules_proto_grpc:repositories.bzl", "rules_proto_grpc_repositories")
load("//bazel/toolchains/rust:repositories.bzl", "rust_repositories")

def toolchain_repositories():
    nasm_repositories()
    perl_repositories()
    python_repositories()
    rules_proto_grpc_repositories()
    rust_repositories()

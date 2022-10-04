load("//bazel/toolchains/nasm:nasm_repositories.bzl", "nasm_repositories")
load("//bazel/toolchains/python:python_repositories.bzl", "python_repositories")
load("//bazel/toolchains/perl:perl_repositories.bzl", "perl_repositories")

def toolchain_repositories():
    nasm_repositories()
    perl_repositories()
    python_repositories()

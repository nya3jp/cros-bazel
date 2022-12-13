load("@rules_perl//perl:deps.bzl", "perl_register_toolchains", "perl_rules_dependencies")
load("@rules_proto_grpc//:repositories.bzl", "rules_proto_grpc_repos", "rules_proto_grpc_toolchains")
load("@rules_proto_grpc//go:repositories.bzl", rules_proto_grpc_go_repos = "go_repos")
load("//bazel/toolchains/rust:toolchains.bzl", "rust_toolchains")

# Must be seperated from language_repositories, as the loads above will fail if they haven't been downloaded yet.

def load_toolchains():
    perl_rules_dependencies()
    perl_register_toolchains()

    native.register_toolchains("//bazel/toolchains/python:python_toolchain")

    rust_toolchains()

    rules_proto_grpc_toolchains()
    rules_proto_grpc_repos()

    # Will need to do this for each language we intend to use rules_proto_grpc
    # with.
    rules_proto_grpc_go_repos()

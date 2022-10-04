load("@rules_perl//perl:deps.bzl", "perl_register_toolchains", "perl_rules_dependencies")

# Must be seperated from language_repositories, as the loads above will fail if they haven't been downloaded yet.

def load_toolchains():
    perl_rules_dependencies()
    perl_register_toolchains()

    native.register_toolchains("//bazel/toolchains/python:python_toolchain")

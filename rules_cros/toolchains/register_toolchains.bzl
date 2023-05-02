load("@rules_perl//perl:deps.bzl", "perl_register_toolchains", "perl_rules_dependencies")
load("//rules_cros/toolchains/rust:toolchains.bzl", "rust_toolchains")

# Must be seperated from language_repositories, as the loads above will fail if they haven't been downloaded yet.

def load_toolchains():
    perl_rules_dependencies()
    perl_register_toolchains()

    rust_toolchains()

load("@rules_rust//rust/private:providers.bzl", "CrateInfo", "DepInfo")
load("@rules_rust//rust/private:rustc.bzl", "collect_deps")
load("@rules_rust//rust/private:utils.bzl", "transform_deps")

def _rust_prebuilt_impl(ctx):
    deps = transform_deps(ctx.attr.deps)
    proc_macro_deps = transform_deps(ctx.attr.proc_macro_deps)

    crate_info = CrateInfo(
        aliases = ctx.attr.aliases,
        compile_data = depset([]),
        compile_data_targets = depset([]),
        deps = depset(deps),
        edition = ctx.attr.edition,
        is_test = False,
        metadata = None,
        name = ctx.label.name,
        output = ctx.file.src,
        owner = None,
        proc_macro_deps = depset(proc_macro_deps),
        root = None,
        rustc_env = {},
        rustc_env_files = [],
        srcs = depset([]),
        # This appears to work for proc-macros as well, which are .so files
        # rather than .rlib files.
        type = "rlib",
        wrapped_crate_type = None,
    )

    dep_info, _, _ = collect_deps(
        crate_info.deps,
        crate_info.proc_macro_deps,
        ctx.attr.aliases,
    )

    return [
        crate_info,
        dep_info,
    ]

rust_prebuilt = rule(
    implementation = _rust_prebuilt_impl,
    doc = "A prebuilt .rlib file (for rust libraries) or a .so file (for proc macros)",
    attrs = dict(
        aliases = attr.label_keyed_string_dict(),
        deps = attr.label_list(),
        proc_macro_deps = attr.label_list(cfg = "exec", providers = [CrateInfo]),
        src = attr.label(allow_single_file = True),
        edition = attr.string(default = "2021")
    ),
    provides = [CrateInfo, DepInfo],
)

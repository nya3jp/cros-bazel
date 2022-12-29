load("@rules_rust//crate_universe:defs.bzl", "crates_vendor")

def _generate_cargo_toml_impl(ctx):
    members = []
    deps = []
    for crate_name, target in ctx.attr.crates.items():
        package = Label(target).package
        members.append("    \"%s\"," % package)
        deps.append("%s = { path = \"%s\" }" % (crate_name, package))
    ctx.actions.expand_template(
        template = ctx.files.tmpl[0],
        output = ctx.outputs.out,
        substitutions = {
            "{WORKSPACE_MEMBERS}": "\n".join(members),
            "{WORKSPACE_DEPS}": "\n".join(deps),
        },
    )

generate_cargo_toml = rule(
    implementation = _generate_cargo_toml_impl,
    attrs = dict(
        tmpl = attr.label(allow_single_file = True, mandatory = True),
        out = attr.output(),
        crates = attr.string_dict(mandatory = True),
    ),
)

def update_crates(name, root_manifest, crates, **kwargs):
    manifests = [root_manifest]
    for crate in crates.values():
        package = crate.split(":")[0]
        manifests.append(crate.split(":")[0] + ":Cargo.toml")

    crates_vendor(
        name = name,
        manifests = manifests,
        **kwargs
    )

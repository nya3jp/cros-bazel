load("//bazel/ebuild/private/common/mountsdk:mountsdk.bzl", "COMMON_ATTRS", "debuggable_mountsdk", "mountsdk_generic")

def _build_image_impl(ctx):
    output = ctx.actions.declare_file(ctx.label.name + ".bin")
    return mountsdk_generic(
        ctx,
        progress_message_name = ctx.label.name,
        inputs = [],
        binpkg_output_file = output,
        outputs = [output],
        args = ctx.actions.args(),
    )

_build_image = rule(
    implementation = _build_image_impl,
    attrs = dict(
        _builder = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/build_image"),
        ),
        **COMMON_ATTRS
    ),
)

def build_image(name, **kwargs):
    debuggable_mountsdk(name = name, orig_rule = _build_image, **kwargs)

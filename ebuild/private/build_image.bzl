load("//bazel/ebuild/private:common.bzl", "OverlaySetInfo")
load("//bazel/ebuild/private/common/mountsdk:mountsdk.bzl", "COMMON_ATTRS", "debuggable_mountsdk", "mountsdk_generic")

def _build_image_impl(ctx):
    output = ctx.actions.declare_file(ctx.label.name + ".bin")

    args = ctx.actions.args()
    direct_inputs = []

    for overlay in ctx.attr.overlays[OverlaySetInfo].overlays:
        args.add("--overlay=%s=%s" % (overlay.mount_path, overlay.squashfs_file.path))
        direct_inputs.append(overlay.squashfs_file)

    return mountsdk_generic(
        ctx,
        progress_message_name = ctx.label.name,
        inputs = direct_inputs,
        binpkg_output_file = output,
        outputs = [output],
        args = args,
    )

_build_image = rule(
    implementation = _build_image_impl,
    attrs = dict(
        _builder = attr.label(
            executable = True,
            cfg = "exec",
            default = Label("//bazel/ebuild/private/cmd/build_image"),
        ),
        overlays = attr.label(
            providers = [OverlaySetInfo],
        ),
        **COMMON_ATTRS
    ),
)

def build_image(name, **kwargs):
    debuggable_mountsdk(name = name, orig_rule = _build_image, **kwargs)

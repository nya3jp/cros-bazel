load("@bazel_skylib//lib:paths.bzl", "paths")


PackageInfo = provider(
  "Portage package info",
  fields = {
    "squashfs_file": "File",
    "build_target_deps": "Depset",
    "runtime_deps": "Depset",
  },
)

OverlayInfo = provider(
  "Portage overlay info",
  fields = {
    "squashfs_file": "File",
  },
)

OverlaySetInfo = provider(
  "Portage overlay set info",
  fields = {
    "squashfs_files": "Depset",
  },
)


def _workspace_root(label):
  return paths.join("..", label.workspace_name) if label.workspace_name else ""


def _relative_path_in_package(file):
  owner = file.owner
  if owner == None:
    fail("File does not have an associated owner label")
  return paths.relativize(file.short_path, paths.join(_workspace_root(owner), owner.package))


def _format_file_arg(file):
  return "--file=%s=%s" % (_relative_path_in_package(file), file.path)


def _ebuild_impl(ctx):
  src_basename = ctx.file.src.basename.rsplit(".", 1)[0]
  output = ctx.actions.declare_file(src_basename + ".squashfs")

  args = ctx.actions.args()
  args.add_all([
    "--ebuild=" + ctx.file.src.path,
    "--category=" + ctx.attr.category,
    "--output=" + output.path,
    "--sdk=" + ctx.file._sdk.path,
  ])

  direct_inputs = [
    ctx.file.src,
    ctx.file._sdk,
  ]
  transitive_inputs = []

  for file in ctx.attr.files:
    transitive_inputs.append(file.files)
    args.add_all(file.files, map_each=_format_file_arg)

  for distfile, name in ctx.attr.distfiles.items():
    files = distfile.files.to_list()
    if len(files) != 1:
      fail("cannot refer to multi-file rule in distfiles")
    file = files[0]
    args.add("--distfile=%s=%s" % (name, file.path))
    direct_inputs.append(file)

  overlay_deps = ctx.attr.overlays[OverlaySetInfo].squashfs_files
  args.add_all(overlay_deps, format_each="--overlay=%s")
  transitive_inputs.append(overlay_deps)

  build_target_deps = depset(
    [dep[PackageInfo].squashfs_file for dep in ctx.attr.build_target_deps]
  )
  runtime_deps = depset(
    [dep[PackageInfo].squashfs_file for dep in ctx.attr.runtime_deps],
    transitive = [dep[PackageInfo].runtime_deps for dep in ctx.attr.runtime_deps]
  )

  args.add_all(build_target_deps, format_each='--dependency=%s')

  transitive_inputs.extend([build_target_deps])

  ctx.actions.run(
    inputs = depset(direct_inputs, transitive = transitive_inputs),
    outputs = [output],
    executable = ctx.executable._tool,
    arguments = [args],
    mnemonic = "Ebuild",
    progress_message = "Building " + ctx.file.src.basename,
  )
  return [
    DefaultInfo(files = depset([output])),
    PackageInfo(
      squashfs_file = output,
      build_target_deps = build_target_deps,
      runtime_deps = runtime_deps,
    ),
  ]


ebuild = rule(
  implementation = _ebuild_impl,
  attrs = {
    "src": attr.label(
      mandatory = True,
      allow_single_file = [".ebuild"],
    ),
    "category": attr.string(
      mandatory = True,
    ),
    "distfiles": attr.label_keyed_string_dict(
      allow_files = True,
    ),
    "build_target_deps": attr.label_list(
      providers = [PackageInfo],
    ),
    "runtime_deps": attr.label_list(
      providers = [PackageInfo],
    ),
    "files": attr.label_list(
      allow_files = True,
    ),
    "overlays": attr.label(
      mandatory = True,
      providers = [OverlaySetInfo],
      cfg = "exec",
    ),
    "_tool": attr.label(
      executable = True,
      cfg = "exec",
      default = Label("//ebuild/private:build_ebuild"),
    ),
    "_sdk": attr.label(
      allow_single_file = True,
      cfg = "exec",
      default = Label("//sdk:squashfs"),
    ),
  },
)


def _format_create_squashfs_arg(file):
  return "%s:%s" % (_relative_path_in_package(file), file.path)


def _create_squashfs_action(ctx, out, exe, files):
  args = ctx.actions.args()
  args.add_all(files, map_each=_format_create_squashfs_arg)
  args.set_param_file_format("multiline")
  args.use_param_file("--specs-from=%s", use_always=True)

  ctx.actions.run(
    inputs = [exe] + files,
    outputs = [out],
    executable = exe.path,
    arguments = ["--output=" + out.path, args],
  )


def _overlay_impl(ctx):
  out = ctx.actions.declare_file(ctx.attr.name + ".squashfs")

  _create_squashfs_action(ctx, out, ctx.executable._create_squashfs, ctx.files.srcs)

  return [
    DefaultInfo(files = depset([out])),
    OverlayInfo(squashfs_file = out),
  ]


overlay = rule(
  implementation = _overlay_impl,
  attrs = {
    "srcs": attr.label_list(
      allow_files = True,
      mandatory = True,
    ),
    "_create_squashfs": attr.label(
      executable = True,
      cfg = "exec",
      default = Label("//ebuild/private:create_squashfs"),
    ),
  },
)


def _overlay_set_impl(ctx):
  return [
    OverlaySetInfo(squashfs_files = depset([
      overlay[OverlayInfo].squashfs_file
      for overlay in ctx.attr.overlays
    ], order = "preorder")),
  ]


overlay_set = rule(
  implementation = _overlay_set_impl,
  attrs = {
    "overlays": attr.label_list(
      providers = [OverlayInfo],
      cfg = "exec",
    ),
  },
)

PackageInfo = provider(
  "Portage package info",
  fields = {
    "squashfs_file": "File",
    "build_host_deps": "Depset",
    "build_target_deps": "Depset",
    "runtime_deps": "Depset",
  },
)


def _ebuild_impl(ctx):
  src_basename = ctx.file.src.basename.rsplit(".", 1)[0]
  output = ctx.actions.declare_file(src_basename + ".squashfs")
  args = ctx.actions.args()
  args.add_all([
    "--ebuild=" + ctx.file.src.path,
    "--category=" + ctx.attr.category,
    "--output=" + output.path,
    "--overlay-squashfs=" + ctx.file._sdk.path,
  ])
  direct_inputs = [
    ctx.file.src,
    ctx.file._sdk,
  ]

  for distfile, name in ctx.attr.distfiles.items():
    files = distfile.files.to_list()
    if len(files) != 1:
      fail("cannot refer to multi-file rule in distfiles")
    file = files[0]
    args.add("--distfile=%s=%s" % (name, file.path))
    direct_inputs.append(file)

  for file in ctx.files.eclasses:
    args.add("--eclass=" + file.path)
    direct_inputs.append(file)

  build_host_deps = depset(
    [dep[PackageInfo].squashfs_file for dep in ctx.attr.build_host_deps],
    transitive = [dep[PackageInfo].runtime_deps for dep in ctx.attr.build_host_deps]
  )
  build_target_deps = depset(
    [dep[PackageInfo].squashfs_file for dep in ctx.attr.build_target_deps]
  )
  runtime_deps = depset(
    [dep[PackageInfo].squashfs_file for dep  in ctx.attr.runtime_deps],
    transitive = [dep[PackageInfo].runtime_deps for dep in ctx.attr.runtime_deps]
  )

  args.add_all(build_host_deps, format_each='--overlay-squashfs=%s')
  # TODO: Support target deps
  # args.add_all(build_target_deps, format_each='--mount=/build/target/=%s')

  inputs = depset(
    direct_inputs,
    transitive = [build_host_deps, build_target_deps]
  )
  outputs = [
    output,
  ]
  ctx.actions.run(
    inputs = inputs,
    outputs = outputs,
    executable = ctx.executable._tool,
    arguments = [args],
    mnemonic = "Ebuild",
    progress_message = "Building " + ctx.file.src.basename,
  )
  return [
    DefaultInfo(files = depset([output])),
    PackageInfo(
      squashfs_file = output,
      build_host_deps = build_host_deps,
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
    "eclasses": attr.label_list(
      allow_files = True,
    ),
    "build_host_deps": attr.label_list(
      providers = [PackageInfo],
      cfg = "exec",
    ),
    "build_target_deps": attr.label_list(
      providers = [PackageInfo],
    ),
    "runtime_deps": attr.label_list(
      providers = [PackageInfo],
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

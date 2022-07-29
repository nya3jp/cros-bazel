def _ebuild_impl(ctx):
  out = ctx.actions.declare_file(ctx.file.src.basename.rsplit(".", 1)[0] + ".tbz2")
  inputs = [ctx.file.src]
  arguments = [
    "--ebuild=" + ctx.file.src.path,
    "--category=" + ctx.attr.category,
    "--output=" + out.path,
  ]
  for distfile, name in ctx.attr.distfiles.items():
    files = distfile.files.to_list()
    if len(files) != 1:
      fail("cannot refer to multi-file rule in distfiles")
    file = files[0]
    inputs.append(file)
    arguments.append("--distfile=%s=%s" % (name, file.path))
  ctx.actions.run(
    inputs = inputs,
    outputs = [out],
    executable = ctx.executable._tool,
    arguments = arguments,
    mnemonic = "Ebuild",
    progress_message = "Building " + ctx.file.src.basename,
  )
  return [DefaultInfo(files = depset([out]))]


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
    "_tool": attr.label(
      executable = True,
      cfg = "exec",
      default = Label("//ebuild/private:build_ebuild"),
    ),
  },
)

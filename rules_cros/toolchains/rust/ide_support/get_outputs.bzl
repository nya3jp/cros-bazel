# Copyright 2023 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Unfortunately, this DFS runs per target (and we cannot control that), so this
# could potentially get very slow if the user runs it on //... or similar.

# In that example, if crate foo depends on crate bar, bar would run dfs(bar) -> [bar, transitive_deps(bar)]

def format(target):
  crate_infos = [v for k, v in providers(target).items() if k.endswith("%CrateInfo")]
  seen = {k: True for k in crate_infos}

  outputs = {}
  # Bazel doesn't allow infinite loops, so this is instead of "while True"
  # TODO: try setting this to 1. Does it affect things
  # TODO: can we make the output format instead a mapping of source file -> [rustc file], and then on_save only selects the latest per source file.
  for i in range(99999999):
    if not crate_infos:
      break
    crate_info = crate_infos.pop()

    rustc_output = crate_info.rust_lib_rustc_output
    if rustc_output != None:
      outputs[crate_info.root.path] = rustc_output.path
  
    deps = crate_info.deps.to_list()
    deps.extend(crate_info.proc_macro_deps.to_list())
    for dep in deps:
      if not seen.get(dep.crate_info) and dep.crate_info != None:
        crate_infos.append(dep.crate_info)
        seen[dep.crate_info] = True
  
  return json.encode(outputs)
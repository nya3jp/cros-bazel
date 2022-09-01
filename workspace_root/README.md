This directory contains `WORKSPACE.bazel`/`BUILD.bazel` to be placed at `src`
directory in a ChromeOS checkout.

We use `linkfile` directive in repo manifests to create symlinks automatically
on checking out. See `.../manifest/_bazel.xml` for details.

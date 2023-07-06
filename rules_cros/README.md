# Chrome OS Bazel Rules

## Overview

This repository provides common rules and configuration for Bazel projects in
Chrome OS.

## Documentation

The code is this directory is experimental and under active development.
Stability and functionality is not guaranteed :)

### Configuring dev experience
#### Rust
We use rust_analyzer to generate a `rust-project.json` file in the workspace root, which allows vscode to understand your dependencies.

Simply install the [rust_analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) VSCode plugin, and then add the following to your `.vscode/tasks.json` file, substituting TARGET for a label of any bazel target you want to depend on. VSCode will then analyze all the transitive dependencies of the specified targets.

For example, I might use the following line:

`"args": ["run", "//bazel/rust:gen_rust_project", "--", "//bazel/rust/examples/use_local_crate/...", "//bazel/rust/examples/hello_world:helllo_world"],`

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Generate rust-project.json",
            "command": "bazel",
            "args": ["run", "//bazel/rust:gen_rust_project", "--", "<TARGET1>", "<TARGET2>", "..."],
            "options": {
                "cwd": "${workspaceFolder}"
            },
            "group": "build",
            "problemMatcher": [],
            "presentation": {
                "reveal": "never",
                "panel": "dedicated",
            },
            "runOptions": {
                "runOn": "folderOpen"
            }
        },
    ]
}
```

If you add a new file, you may need to manually re-run the task.

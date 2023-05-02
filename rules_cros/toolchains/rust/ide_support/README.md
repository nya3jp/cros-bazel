# Setting up vscode for rust development

## Step 1: Install bazel-rust-analyzer vscode extension
Install [https://marketplace.visualstudio.com/items?itemName=MattStark.bazel-rust-analyzer](bazel-rust-analyzer) extension.

## Step 2 (optional): Install ibazel
* See [ibazel](https://github.com/bazelbuild/bazel-watcher) (automatically reruns bazel when files change).

## Step 3: Ensure your workspace is configured correctly
Rustc outputs paths relative to `src`, and vscode attempts to interpret paths as relative to the first workspace root.

Thus, for now, this only works with single-folder workspaces, and if you're having performance problems with vscode, use search.exclude to prevent vscode from indexing or searching directories you don't care about. The following workspace configuration is a good start.
```
{
	"folders": [
		{
			"name": "src
			"path": "."
		},
        ],
	"settings": {
                "files.exclude": {
                },
                "search.exclude": {
                        "bazel-bin/**": true,
                        "bazel-out/**": true,
                        "bazel-src/**": true,
                        "bazel-testlogs/**": true,
                },
	}
}
```

## Step 4: Add rust-analyzer config
Add the following configuration to your vscode's settings.json.

```
"rust-analyzer.linkedProjects": ["${workspaceFolder}/rust-project.json"],
"rust-analyzer.check.overrideCommand": ["rules_cros/toolchains/rust/ide_support/flycheck.sh"],
"bazel-rust-analyzer.genRustProjectCommand": [
        "env",
        "RUST_LOG=info",
        "bazel",
        "run",
        "//rules_cros/toolchains/rust/ide_support:gen_rust_project",
        "--",
        "--files"
],
"bazel-rust-analyzer.autoGenRustProjectCommand": true,
```

# Using the configured vscode
Follow the usage instructions instructions for the [https://marketplace.visualstudio.com/items?itemName=MattStark.bazel-rust-analyzer](bazel-rust-analyzer) extension.

You can also use bazel to make the error messages automatically refresh on save.
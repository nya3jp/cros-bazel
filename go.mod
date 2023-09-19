module cros.local/bazel

go 1.18

// To update:
// 1) Update this file
// 2) Run "go mod tidy"
// 3) Add / remove from use_repo in MODULE.bazel

require (
	github.com/alessio/shellescape v1.4.1
	github.com/bazelbuild/rules_go v0.38.1
	github.com/elastic/go-seccomp-bpf v1.2.0
	github.com/google/go-cmp v0.5.9
	github.com/urfave/cli/v2 v2.20.3
	golang.org/x/net v0.0.0-20210405180319-a5a99cb37ef4
	golang.org/x/sys v0.4.0
)

require (
	github.com/cpuguy83/go-md2man/v2 v2.0.2 // indirect
	github.com/pkg/errors v0.8.1 // indirect
	github.com/russross/blackfriday/v2 v2.1.0 // indirect
	github.com/xrash/smetrics v0.0.0-20201216005158-039620a65673 // indirect
)

module cros.local/bazel

go 1.18

require (
	github.com/alecthomas/participle/v2 v2.0.0-beta.5
	github.com/alessio/shellescape v1.4.1
	github.com/elastic/go-seccomp-bpf v1.2.0
	github.com/google/go-cmp v0.5.8
	github.com/klauspost/compress v1.15.12
	github.com/urfave/cli/v2 v2.20.3
	golang.org/x/net v0.0.0-20210405180319-a5a99cb37ef4
	golang.org/x/sys v0.0.0-20220811171246-fbc7d0a398ab
	google.golang.org/grpc v1.50.1
	google.golang.org/protobuf v1.28.0
	mvdan.cc/sh/v3 v3.5.1
)

require (
	// WARNING: Keep the rules_go version here in sync with the other one
	// specified in ebuild/repositories.bzl!
	// On building our Go code with Bazel, rules_go is loaded before
	// bazel_gazelle, thus the version specification here is ignored.
	github.com/bazelbuild/rules_go v0.36.0
	github.com/cpuguy83/go-md2man/v2 v2.0.2 // indirect
	github.com/golang/mock v1.6.0 // indirect
	github.com/golang/protobuf v1.5.2 // indirect
	github.com/pkg/errors v0.8.1 // indirect
	github.com/russross/blackfriday/v2 v2.1.0 // indirect
	github.com/xrash/smetrics v0.0.0-20201216005158-039620a65673 // indirect
)

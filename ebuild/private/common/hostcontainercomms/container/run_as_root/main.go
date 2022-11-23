package main

import (
	"context"
	"os"

	"cros.local/bazel/ebuild/private/common/hostcontainercomms/container"
)

func main() {
	container.RootExec(context.Background(), os.Args[1], os.Args[2:]...)
}

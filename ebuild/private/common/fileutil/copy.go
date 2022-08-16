package fileutil

import (
	"os"
	"os/exec"
)

func Copy(src, dst string) error {
	cmd := exec.Command("/usr/bin/cp", "--", src, dst)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

package runfiles

import (
	"fmt"
	"os"
)

func FixEnv() {
	const envName = "RUNFILES_DIR"
	if os.Getenv(envName) != "" {
		return
	}

	exe, err := os.Executable()
	if err != nil {
		panic(fmt.Sprintf("fixing environment variables for runfiles access: %v", err))
	}

	os.Setenv(envName, exe+".runfiles")
}

package portagevars

import (
	"strings"

	"cros.local/ebuild/private/common/standard/makevars"
)

func Overlays(vars makevars.Vars) []string {
	return append([]string{vars["PORTDIR"]}, strings.Fields(vars["PORTDIR_OVERLAY"])...)
}

package portagevars

import (
	"strings"

	"cros.local/rules_ebuild/ebuild/private/common/standard/makevars"
)

func Overlays(vars makevars.Vars) []string {
	return append([]string{vars["PORTDIR"]}, strings.Fields(vars["PORTDIR_OVERLAY"])...)
}

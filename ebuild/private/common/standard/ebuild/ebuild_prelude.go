package ebuild

import _ "embed"

//go:embed ebuild_prelude.sh
var preludeCode []byte

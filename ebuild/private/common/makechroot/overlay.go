package makechroot

import (
	"fmt"
	"strings"
)

type OverlayInfo struct {
	MountDir     string
	SquashfsPath string
}

func ParseOverlaySpecs(specs []string) ([]OverlayInfo, error) {
	var overlays []OverlayInfo
	for _, spec := range specs {
		v := strings.Split(spec, "=")
		if len(v) != 2 {
			return nil, fmt.Errorf("invalid overlay spec: %s", spec)
		}
		overlays = append(overlays, OverlayInfo{
			MountDir:     strings.Trim(v[0], "/"),
			SquashfsPath: v[1],
		})
	}
	return overlays, nil
}

package mountsdk

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/urfave/cli/v2"
)

var flagSDK = &cli.StringSliceFlag{
	Name:     "sdk",
	Required: true,
}

var flagOverlay = &cli.StringSliceFlag{
	Name:     "overlay",
	Required: true,
	Usage: "<inside path>=<squashfs file | directory | tar.*>: " +
		"Mounts the file or directory at the specified path. " +
		"Inside path can be absolute or relative to /mnt/host/source/.",
}

var CLIFlags = []cli.Flag{
	flagSDK,
	flagOverlay,
}

func GetMountConfigFromCLI(c *cli.Context) (*Config, error) {
	cfg := Config{}

	for _, sdk := range c.StringSlice(flagSDK.Name) {
		cfg.Overlays = append(cfg.Overlays, MappedDualPath{HostPath: sdk, SDKPath: "/"})
	}

	for _, spec := range c.StringSlice(flagOverlay.Name) {
		v := strings.Split(spec, "=")
		if len(v) != 2 {
			return nil, fmt.Errorf("invalid Overlay spec: %s", spec)
		}

		overlay := MappedDualPath{
			HostPath: v[1],
			SDKPath:  strings.TrimSuffix(v[0], "/"),
		}
		if !filepath.IsAbs(overlay.SDKPath) {
			overlay.SDKPath = filepath.Join(SourceDir, overlay.SDKPath)
		}
		cfg.Overlays = append(cfg.Overlays, overlay)
	}
	return &cfg, nil
}

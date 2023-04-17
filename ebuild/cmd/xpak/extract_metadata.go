// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package main

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"text/template"

	"cros.local/bazel/ebuild/private/common/bazelutil"
	"cros.local/bazel/ebuild/private/common/portage/binarypackage"
	"cros.local/bazel/ebuild/private/common/portage/dependency"
	"cros.local/bazel/ebuild/private/common/tar"
)

var (
	repoNameToPath = map[string]string{
		"cros":           "third_party/chromiumos-overlay",
		"portage-stable": "third_party/portage-stable",
	}
)

type Needed struct {
	arch       string
	absLibPath string
	soname     string
	rpaths     []string
	needed     []string
	multilib   string
}

type packageMetadata struct {
	overlayPath string
	atom        *dependency.Atom
	needed      []Needed
	files       []tar.FileListItem
	sharedLibs  []string
	// The list of .so's required by the shared libs defined in `sharedLibs`.
	sharedLibDeps []string
	// The list of .so's required by the ELF binaries
	binLibDeps []string
}

var sharedLibRegex = regexp.MustCompile(
	`^(?P<linkername>\w+\.so)(:?\.(?P<major>\d+)(:?\.(?P<minor>\d+)(:?\.(?P<rev>\d+))?)?)?$`)

// Ignore libs provided by glibc
func ignoredSharedLib(name string) bool {
	if name == "" {
		return true
	} else if strings.HasPrefix(name, "ld-linux-") {
		return true
	} else if strings.HasPrefix(name, "libc.so.") {
		return true
	} else if strings.HasPrefix(name, "libpthread.so.") {
		return true
	}
	return false
}

func newPackageMetadata(overlayName string, atom *dependency.Atom, needed []Needed, files []tar.FileListItem) (*packageMetadata, error) {
	for _, file := range files {
		if file.Path == "" {
			return nil, fmt.Errorf("File path is empty!")
		}

		if !strings.HasPrefix(file.Path, "./") {
			return nil, fmt.Errorf("File path is not relative: %s", file.Path)
		}
	}

	var sharedLibs []string
	sharedLibDeps := make(map[string]struct{})
	binLibDeps := make(map[string]struct{})
	providedSoNames := make(map[string]struct{})
	// Verify all packages produce a fully qualified version number.
	for _, lib := range needed {
		baseName := filepath.Base(lib.absLibPath)

		matches := sharedLibRegex.FindStringSubmatch(baseName)
		if matches == nil { // ELF binary
			for _, dep := range lib.needed {
				if !ignoredSharedLib(dep) {
					binLibDeps[dep] = struct{}{}
				}
			}
			continue
		} else { // .so library
			if len(matches) != 8 {
				return nil, fmt.Errorf("%s must have the following format: libX.so.X.X.X", lib.absLibPath)
			}

			var soName string
			if matches[3] != "" {
				soName = fmt.Sprintf("%s.%s", matches[1], matches[3])
			} else {
				// Some .so's aren't versioned
				soName = matches[1]
			}

			providedSoNames[soName] = struct{}{}

			sharedLibs = append(sharedLibs, lib.absLibPath)
			for _, dep := range lib.needed {
				if !ignoredSharedLib(dep) {
					sharedLibDeps[dep] = struct{}{}
				}
			}
		}
	}

	sharedLibDepsList := make([]string, 0, len(sharedLibDeps))
	for dep := range sharedLibDeps {
		if _, ok := providedSoNames[dep]; ok {
			// Don't list provided so's as deps
			continue
		}
		sharedLibDepsList = append(sharedLibDepsList, dep)
	}
	sort.Strings(sharedLibDepsList)

	binLibDepsList := make([]string, 0, len(binLibDeps))
	for dep := range binLibDeps {
		if _, ok := providedSoNames[dep]; ok {
			// Don't list provided so's as deps
			continue
		}
		if _, ok := sharedLibDeps[dep]; ok {
			// dep is already listed
			continue
		}
		binLibDepsList = append(binLibDepsList, dep)
	}
	sort.Strings(binLibDepsList)

	overlayPath, ok := repoNameToPath[overlayName]
	if !ok {
		return nil, fmt.Errorf("Unknown mapping for overlay: %s", overlayName)
	}

	return &packageMetadata{overlayPath, atom, needed, files, sharedLibs, sharedLibDepsList, binLibDepsList}, nil
}

func (m packageMetadata) IsEmpty() bool {
	return (len(m.Headers()) == 0 &&
			len(m.PkgConfig()) == 0 &&
			len(m.StaticLibs()) == 0 &&
			len(m.SharedLibs()) == 0)
}

func (m packageMetadata) EbuildPath() string {
	name := strings.Split(m.atom.PackageName(), "/")[1]

	return fmt.Sprintf("%s/%s/%s-%s.ebuild", m.overlayPath, m.atom.PackageName(), name, m.atom.Version())
}

func (m packageMetadata) BinPkgFileName() string {
	name := strings.Split(m.atom.PackageName(), "/")[1]

	return fmt.Sprintf("%s-%s.tbz2", name, m.atom.Version())
}

func (m packageMetadata) Headers() []string {
	var items []string
	for _, file := range m.files {
		if !strings.Contains(file.Path, "/include/") {
			continue
		}

		items = append(items, file.Path[1:])
	}
	return items
}

func (m packageMetadata) PkgConfig() []string {
	var items []string
	for _, file := range m.files {
		if !strings.HasSuffix(file.Path, ".pc") {
			continue
		}

		items = append(items, file.Path[1:])
	}
	return items
}

func (m packageMetadata) StaticLibs() []string {
	var items []string
	for _, file := range m.files {
		if !strings.HasSuffix(file.Path, ".a") {
			continue
		}

		items = append(items, file.Path[1:])
	}
	return items
}

func (m packageMetadata) SharedLibs() []string {
	return m.sharedLibs
}

func (m packageMetadata) SharedLibDeps() []string {
	return m.sharedLibDeps
}

func (m packageMetadata) BinLibDeps() []string {
	return m.binLibDeps
}

func parseNeeded(xpak binarypackage.XPAK) ([]Needed, error) {
	raw, ok := xpak["NEEDED.ELF.2"]
	if !ok {
		// Not all packages have shared objects
		return nil, nil
	}
	content := string(raw)

	lines := strings.Split(content, "\n")
	items := make([]Needed, 0, len(lines))
	for _, line := range lines {
		if line == "" {
			continue
		}

		item := Needed{}

		v := strings.Split(line, ";")
		if len(v) < 5 {
			return nil, fmt.Errorf("Expected at least 5 columns, got %d: %s", len(v), line)
		}
		item.arch = v[0]
		item.absLibPath = v[1]
		item.soname = v[2]
		item.rpaths = strings.Split(v[3], ":")
		item.needed = strings.Split(v[4], ",")

		if len(v) > 5 {
			// This field is optional
			item.multilib = v[5]
		}

		items = append(items, item)
	}

	return items, nil
}

func collectContents(binPkg *binarypackage.File) ([]tar.FileListItem, error) {
	reader, err := binPkg.TarballReader()
	if err != nil {
		return nil, err
	}
	defer reader.Close()

	return tar.ListFilesZstd(reader)
}

func extractMetadata(ctx context.Context, fileName string) (*packageMetadata, error) {
	binPkg, err := binarypackage.BinaryPackage(fileName)
	if err != nil {
		return nil, fmt.Errorf("failed opening binpkg: %w", err)
	}
	defer binPkg.Close()

	xpakHeader, err := binPkg.Xpak()
	if err != nil {
		return nil, err
	}

	pkgNameBytes, ok := xpakHeader["PF"]
	if !ok {
		return nil, fmt.Errorf("failed to read PF key: %s", fileName)
	}
	pkgName := strings.TrimSpace(string(pkgNameBytes))

	categoryBytes, ok := xpakHeader["CATEGORY"]
	if !ok {
		return nil, fmt.Errorf("failed to read CATEGORY key: %s", fileName)
	}
	category := strings.TrimSpace(string(categoryBytes))

	atom, err := dependency.ParseAtom(fmt.Sprintf("=%s/%s", category, pkgName))
	if err != nil {
		return nil, err
	}

	overlayBytes, ok := xpakHeader["repository"]
	if !ok {
		return nil, fmt.Errorf("failed to read repository key: %s", fileName)
	}
	overlay := strings.TrimSpace(string(overlayBytes))

	needed, err := parseNeeded(xpakHeader)
	if err != nil {
		return nil, err
	}

	contents, err := collectContents(binPkg)
	if err != nil {
		return nil, err
	}

	metadata, err := newPackageMetadata(overlay, atom, needed, contents)
	if err != nil {
		return nil, err
	}

	return metadata, nil
}

var metadataTemplate = template.Must(template.New("").Parse(`# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#
# This file was generated using the following command. Please remove this
# message if the file was manually updated.
# $ bazel run //bazel/ebuild/cmd/xpak:xpak -- extract-metadata {{ .BinPkgFileName }}
#
# When would I manually update this file? If some of the outputs or deps are
# dependent on a USE flag. The generator doesn't have a way of extracting that
# data so it needs to be manually entered.
#
# i.e.,
# OUT_HEADERS="extra?( /usr/include/extra.h )"
#
# Why do we need this file? It's not used by portage, but by our ebuild -> bazel
# BUILD file generator. It allows us to generate an optimized build graph.
#
{{- if .Headers }}
OUT_HEADERS="
  {{- range .Headers }}
  {{ . }}
  {{- end }}
"
{{- end }}
{{- if .PkgConfig }}
OUT_PKG_CONFIG="
  {{- range .PkgConfig }}
  {{ . }}
  {{- end }}
"
{{- end }}
{{- if .StaticLibs }}
OUT_STATIC_LIBS="
  {{- range .StaticLibs }}
  {{ . }}
  {{- end }}
"
{{- end }}
{{- if .SharedLibs }}
OUT_SHARED_LIBS="
  {{- range .SharedLibs }}
  {{ . }}
  {{- end }}
"
{{- end }}
{{- if .SharedLibDeps }}
# The .so's required by all the shared libraries that are produced by this
# package. These will be included as transitive dependencies for all of reverse
# dependencies of this package.
OUT_SHARED_LIB_DEPS="
  {{- range .SharedLibDeps }}
  {{ . }}
  {{- end }}
"
{{- end }}
{{- if .BinLibDeps }}
# The .so's required by all the executable binaries that are produced by this
# package. These will only be included as runtime dependencies (i.e., not build
#	time dependencies) by the reverse dependencies of this package.
#
# If a dependency is specified in OUT_SHARED_LIB_DEPS, it shouldn't be listed
# here.
OUT_BIN_LIB_DEPS="
  {{- range .BinLibDeps }}
  {{ . }}
  {{- end }}
"
{{- end }}
`))

func writeMetadataTemplate(ebuild string, m *packageMetadata) error {
	out, err := os.Create(ebuild + ".ext")
	if err != nil {
		return err
	}
	defer out.Close()

	return metadataTemplate.Execute(out, m)
}

func extractMetadataCmd(ctx context.Context, write bool, fileNames []string) error {
	for _, fileName := range fileNames {
		m, err := extractMetadata(ctx, fileName)
		if err != nil {
			return err
		}

		fmt.Printf("%s\n", m.EbuildPath())

		if len(m.Headers()) > 0 {
			fmt.Printf("  Headers: %s\n", m.Headers())
		}
		if len(m.PkgConfig()) > 0 {
			fmt.Printf("  PkgConfig: %s\n", m.PkgConfig())
		}
		if len(m.StaticLibs()) > 0 {
			fmt.Printf("  StaticLibs: %s\n", m.StaticLibs())
		}
		if len(m.SharedLibs()) > 0 {
			fmt.Printf("  SharedLibs: %s\n", m.SharedLibs())
		}
		if len(m.SharedLibDeps()) > 0 {
			fmt.Printf("  SharedLibDeps: %s\n", m.SharedLibDeps())
		}
		if len(m.BinLibDeps()) > 0 {
			fmt.Printf("  BinLibDeps: %s\n", m.BinLibDeps())
		}

		ebuild := filepath.Join(bazelutil.WorkspaceDir(), m.EbuildPath())

		if m.IsEmpty() {
			// TODO: Should we delete the metadata file if it exists?
			continue
		}

		if _, err := os.Stat(ebuild); err != nil {
			if write {
				return fmt.Errorf("ebuild %s: %w", ebuild, err)
			} else {
				// Just issue a warning
				fmt.Printf("ebuild %s: %v", ebuild, err)
				continue
			}
		}

		if err = writeMetadataTemplate(ebuild, m); err != nil {
			return err
		}

	}
	return nil
}

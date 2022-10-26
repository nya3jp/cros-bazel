// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package srcparse

import (
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"cros.local/bazel/ebuild/private/common/depdata"
	"cros.local/bazel/ebuild/private/common/standard/dependency"
	"cros.local/bazel/ebuild/private/common/standard/packages"
)

type manifestEntry struct {
	Size      int
	Integrity string
	SHA256    string
	SHA512    string
}

func parseSimpleURIs(deps *dependency.Deps, manifest map[string]*manifestEntry) (map[string]*depdata.URIInfo, error) {
	uriMap := make(map[string][]string)
	for _, expr := range deps.Expr().Children() {
		uri, ok := expr.(*dependency.Uri)
		if !ok {
			return nil, fmt.Errorf("Expected Uri, got %s", expr)
		}
		var fileName string
		if uri.FileName() != nil {
			fileName = *uri.FileName()
		} else {
			parsedURI, err := url.ParseRequestURI(uri.Uri())
			if err != nil {
				return nil, err
			}
			fileName = filepath.Base(parsedURI.Path)
		}

		uriMap[fileName] = append(uriMap[fileName], uri.Uri())
	}

	uriInfoMap := make(map[string]*depdata.URIInfo)
	for fileName, uris := range uriMap {
		entry, ok := manifest[fileName]
		if !ok {
			return nil, fmt.Errorf("cannot find file %s in Manifest %v", fileName, manifest)
		}

		uriInfoMap[fileName] = &depdata.URIInfo{
			Uris:      uris,
			Size:      entry.Size,
			Integrity: entry.Integrity,
			// TODO: Remove these when we can use integrity
			SHA256: entry.SHA256,
			SHA512: entry.SHA512,
		}
	}

	return uriInfoMap, nil
}

func hashToIntegrity(name string, hexHash string) (string, error) {
	hashBytes, err := hex.DecodeString(hexHash)
	if err != nil {
		return "", err
	}

	hashBase64 := base64.StdEncoding.EncodeToString(hashBytes)

	integrity := fmt.Sprintf("%s-%s", strings.ToLower(name), hashBase64)

	return integrity, nil
}

func parseManifest(eBuildPath string) (map[string]*manifestEntry, error) {
	files := make(map[string]*manifestEntry)

	ebuildDir := filepath.Dir(eBuildPath)

	// Read Manifest to get a list of distfiles.
	manifest, err := os.ReadFile(filepath.Join(ebuildDir, "Manifest"))
	if err != nil {
		return nil, err
	}

	for _, line := range strings.Split(string(manifest), "\n") {
		fields := strings.Fields(line)
		if len(fields) < 3 || fields[0] != "DIST" {
			continue
		}

		fileName, err := url.PathUnescape(fields[1])
		if err != nil {
			return nil, err
		}

		size, err := strconv.Atoi(fields[2])
		if err != nil {
			return nil, err
		}

		hexHashes := make(map[string]string)
		for i := 3; i+1 < len(fields); i += 2 {
			hexHashes[fields[i]] = fields[i+1]
		}

		// We prefer SHA512 for integrity checking
		for _, hashName := range []string{"SHA512", "SHA256", "BLAKE2B"} {
			hexHash, ok := hexHashes[hashName]
			if ok {
				integrity, err := hashToIntegrity(hashName, hexHash)
				if err != nil {
					return nil, err
				}

				files[fileName] = &manifestEntry{
					Size:      size,
					Integrity: integrity,
					// Our version of bazel doesn't support integrity on http_file, only http_archive
					// so we need to plumb in the hashes.
					SHA256: hexHashes["SHA256"],
					// If we don't have a SHA256 we will use the SHA512 to verify the downloaded file
					// and then compute the SHA256
					SHA512: hexHashes["SHA512"],
				}
				break
			}
		}
	}

	return files, nil
}

func ExtractURIs(pkg *packages.Package) (map[string]*depdata.URIInfo, error) {
	srcURI, ok := pkg.Metadata()["SRC_URI"]
	if ok && srcURI != "" {
		srcURIs, err := dependency.Parse(srcURI)
		if err != nil {
			return nil, err
		}

		srcURIs = dependency.ResolveUse(srcURIs, pkg.Uses())
		srcURIs = dependency.Simplify(srcURIs)

		if len(srcURIs.Expr().Children()) > 0 {
			manifest, err := parseManifest(pkg.Path())
			if err != nil {
				return nil, err
			}

			srcURIInfo, err := parseSimpleURIs(srcURIs, manifest)
			if err != nil {
				return nil, err
			}

			return srcURIInfo, err
		}
	}

	return nil, nil
}

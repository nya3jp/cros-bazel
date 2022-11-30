// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package distfiles

import (
	"crypto/sha256"
	"crypto/sha512"
	"encoding/hex"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/url"
	"path/filepath"
	"regexp"

	"cros.local/bazel/ebuild/private/common/depdata"
)

var allowedHosts = map[string]struct{}{
	"commondatastorage.googleapis.com": {},
	"storage.googleapis.com":           {},
}

var distBaseURLs = []string{
	"https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/",
	"https://commondatastorage.googleapis.com/chromeos-localmirror/distfiles/",
	"https://commondatastorage.googleapis.com/chromeos-localmirror/lvfs/",
	"https://storage.googleapis.com/chromeos-localmirror/secureshell/distfiles/",
	"https://storage.googleapis.com/chromeos-localmirror/secureshell/releases/",
	"https://storage.googleapis.com/chromium-nodejs/14.15.4/",
	"https://storage.googleapis.com/chromium-nodejs/16.13.0/",
}

// The following packages don't exist in the mirrors above, but instead
// need to be pulled from the SRC_URI. We should probably mirror these so our
// build isn't dependent on external hosts.
var manualDistfileMap = map[string]string{
	"iproute2-5.16.0.tar.gz": "http://www.kernel.org/pub/linux/utils/net/iproute2/iproute2-5.16.0.tar.gz",
	"pyftdi-0.54.0.tar.gz": "https://files.pythonhosted.org/packages/source/p/pyftdi/pyftdi-0.54.0.tar.gz",
}

func getSHA(url string) (string, string, error) {
	log.Printf("Trying: %s", url)
	res, err := http.Get(url)
	if err != nil {
		return "", "", err
	}
	defer res.Body.Close()

	if res.StatusCode/100 != 2 {
		return "", "", fmt.Errorf("http status %d", res.StatusCode)
	}

	// TODO: Use io.TeeReader to avoid loading full content into memory.
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return "", "", err
	}

	sha256Hash := sha256.Sum256(body)
	sha512Hash := sha512.Sum512(body)

	// Need to unsize the slice
	sha256HashHex := hex.EncodeToString(sha256Hash[:])
	sha512HashHex := hex.EncodeToString(sha512Hash[:])

	return sha256HashHex, sha512HashHex, nil
}

func Locate(filename string, info *depdata.URIInfo) (*Entry, error) {
	// Once we use bazel 6.0 we can just set integrity value on the
	// http_file.
	var sha256 string
	var sha512 string

	goodURI, ok := manualDistfileMap[filename]
	if !ok {
		for _, uri := range info.Uris {
			parsedURI, err := url.ParseRequestURI(uri)
			if err != nil {
				return nil, err
			}

			if parsedURI.Scheme == "gs" {
				log.Printf("Got gs: %s", parsedURI)
				parsedURI.Scheme = "https"
				parsedURI.Path = filepath.Join(parsedURI.Host, parsedURI.Path)
				parsedURI.Host = "commondatastorage.googleapis.com"
				uri = parsedURI.String()
			} else if parsedURI.Scheme == "cipd" {
				// Don't return an error since it's fatal and we want to keep it that
				// way.
				return nil, nil
			}

			if _, ok := allowedHosts[parsedURI.Host]; !ok {
				continue
			}

			goodURI = uri
			sha256 = info.SHA256
			break
		}
	}

	if goodURI != "" && sha256 == "" {
		// We don't have a SHA256 in the Manifest, Download the file and use
		// the SHA512 in the Manifest to verify the file, then compute the
		// SHA256 from the downloaded file.
		var err error
		sha256, sha512, err = getSHA(goodURI)
		if err != nil {
			return nil, err
		}

		if info.SHA512 != "" && sha512 != info.SHA512 {
			return nil, fmt.Errorf("SHA512 doesn't match for file %s: %s != %s", filename, info.SHA512, sha512)
		}
	} else if goodURI == "" {
		for _, distBaseURL := range distBaseURLs {
			url := distBaseURL + filename

			var err error
			sha256, sha512, err = getSHA(url)
			if err != nil {
				continue
			}

			if info.SHA512 != "" && sha512 != info.SHA512 {
				return nil, fmt.Errorf("SHA512 doesn't match for file %s: %s != %s", filename, info.SHA512, sha512)
			}

			goodURI = url
			break
		}
	}

	log.Printf("Found: %s w/ sha256 %s", goodURI, sha256)

	if goodURI != "" && sha256 != "" {
		name := regexp.MustCompile(`[^A-Za-z0-9]`).ReplaceAllString(filename, "_")
		return &Entry{
			Filename: filename,
			URL:      goodURI,
			SHA256:   sha256,
			Name:     name,
		}, nil
	}
	return nil, fmt.Errorf("no distfile found for %s", filename)
}

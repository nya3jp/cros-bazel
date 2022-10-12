package binarypackage_test

import (
	"bytes"
	"io"
	"os"
	"path/filepath"
	"testing"

	"cros.local/bazel/ebuild/private/common/portage/binarypackage"
	"github.com/bazelbuild/rules_go/go/tools/bazel"
)

const binaryPkgRunfile = "bazel/ebuild/private/common/portage/binarypackage/testdata/nano.tbz2"

// This is the contents of the tarball from the binary package, extracted with
// qtbz2. We are unable to use the qtbz2 binary in bazel, as this would require
// first being able to extract the qtbz2 binary from a package.
const tarballRunfile = "bazel/ebuild/private/common/portage/binarypackage/testdata/nano.tar.bz2"

func TestBinaryPackage(t *testing.T) {
	binaryPkgFile, err := bazel.Runfile(binaryPkgRunfile)
	if err != nil {
		t.Fatal(err)
	}
	tarball, err := bazel.Runfile(tarballRunfile)
	if err != nil {
		t.Fatal(err)
	}

	bp, err := binarypackage.BinaryPackage(binaryPkgFile)
	if err != nil {
		t.Fatalf("Failed to load binary package: %w", err)
	}
	defer bp.Close()

	t.Run("xpak metadata read correctly", func(t *testing.T) {
		xp, err := bp.Xpak()
		if err != nil {
			t.Fatal(err)
		}
		for field, want := range map[string][]byte{
			"CATEGORY":   []byte("app-editors\n"),
			"PF":         []byte("nano-6.4\n"),
			"repository": []byte("portage-stable\n"),
		} {

			if got, exists := xp[field]; exists {
				if !bytes.Equal(want, got) {
					t.Errorf("Incorrect %s: got %v; want %v", field, got, want)
				}
			} else {
				t.Errorf("Field %s doesn't exist", field)
			}
		}
	})

	t.Run("tarball contains correct bytes", func(t *testing.T) {
		want, err := os.ReadFile(tarball)
		if err != nil {
			t.Fatal(err)
		}
		tarballReader, err := bp.TarballReader()
		if err != nil {
			t.Fatalf("Unable to get a tarball reader: %v", err)
		}
		defer tarballReader.Close()
		got, err := io.ReadAll(tarballReader)
		if err != nil {
			t.Fatalf("Unable to read from tarball: %v", err)
		}
		if len(want) != len(got) {
			t.Fatalf("Incorrect number of bytes in tarball: got %d; want %d", len(got), len(want))
		}
		if !bytes.Equal(want, got) {
			// Since these are binary files, actually showing them to the user could
			// be hard.
			t.Fatal("Tarball was the same size, but had different contents to expected")
		}
	})

	t.Run("can merge a package", func(t *testing.T) {
		outDir := t.TempDir()
		if err := os.Mkdir(filepath.Join(outDir, "bin"), 0755); err != nil {
			t.Fatal(err)
		}
		// Put an arbitrary file in the tree to ensure it can merge with
		// existing files.
		if err := os.WriteFile(filepath.Join(outDir, "bin/vim"), []byte{}, 0644); err != nil {
			t.Fatal(err)
		}

		if err := bp.Merge(outDir); err != nil {
			t.Fatalf("Failed to merge package: %v", err)
		}

		st, err := os.Stat(filepath.Join(outDir, "bin/nano"))
		if err != nil {
			t.Fatal(err)
		}
		if st.Mode() != 0755 {
			t.Fatalf("File permissions for bin/nano were incorrect: got %x; want 755", st.Mode())
		}

		st, err = os.Stat(filepath.Join(outDir, "etc/nanorc"))
		if err != nil {
			t.Fatal(err)
		}
		if st.Mode() != 0644 {
			t.Fatalf("File permissions for etc/nanorc were incorrect: got %x; want 644", st.Mode())
		}
	})

	t.Run("fails to merge a package with conflicts", func(t *testing.T) {
		outDir := t.TempDir()
		if err := os.Mkdir(filepath.Join(outDir, "bin"), 0755); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(outDir, "bin/nano"), []byte{}, 0644); err != nil {
			t.Fatal(err)
		}

		if err := bp.Merge(outDir); err == nil {
			t.Fatal("Incorrectly merged a package with conflicts")
		}
	})
}

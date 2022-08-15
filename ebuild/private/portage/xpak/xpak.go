package xpak

import (
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"os"
)

type XPAK map[string][]byte

func Read(path string) (XPAK, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	fi, err := f.Stat()
	if err != nil {
		return nil, err
	}
	size := fi.Size()

	if size < 24 {
		return nil, errors.New("corrupted .tbz2 file: size is too small")
	}
	if err := expectMagic(f, size-4, "STOP"); err != nil {
		return nil, fmt.Errorf("corrupted .tbz2 file: %w", err)
	}
	xpakOffset, err := readUint32(f, size-8)
	if err != nil {
		return nil, fmt.Errorf("corrupted .tbz2 file: %w", err)
	}
	xpakStart := size - 8 - int64(xpakOffset)
	if xpakStart < 0 {
		return nil, errors.New("corrupted .tbz2 file: invalid xpak_offset")
	}
	if err := expectMagic(f, size-16, "XPAKSTOP"); err != nil {
		return nil, fmt.Errorf("corrupted .tbz2 file: %w", err)
	}
	if err := expectMagic(f, xpakStart, "XPAKPACK"); err != nil {
		return nil, fmt.Errorf("corrupted .tbz2 file: %w", err)
	}
	indexLen, err := readUint32(f, xpakStart+8)
	if err != nil {
		return nil, err
	}
	dataLen, err := readUint32(f, xpakStart+12)
	if err != nil {
		return nil, err
	}
	indexStart := xpakStart + 16
	dataStart := indexStart + int64(indexLen)
	if dataStart+int64(dataLen) != size-16 {
		return nil, fmt.Errorf("corrupted .tbz2 file: data length inconsistency")
	}

	xpak := make(map[string][]byte)
	for indexPos := indexStart; indexPos < dataStart; {
		nameLen, err := readUint32(f, indexPos)
		if err != nil {
			return nil, err
		}
		indexPos += 4
		nameBuf := make([]byte, int(nameLen))
		if _, err := io.ReadFull(f, nameBuf); err != nil {
			return nil, err
		}
		indexPos += int64(nameLen)
		name := string(nameBuf)
		dataOffset, err := readUint32(f, indexPos)
		if err != nil {
			return nil, err
		}
		indexPos += 4
		dataLen, err := readUint32(f, indexPos)
		if err != nil {
			return nil, err
		}
		indexPos += 4

		if _, err := f.Seek(dataStart+int64(dataOffset), io.SeekStart); err != nil {
			return nil, err
		}
		data := make([]byte, int(dataLen))
		if _, err := io.ReadFull(f, data); err != nil {
			return nil, err
		}

		xpak[name] = data
	}

	return xpak, nil
}

func readUint32(f *os.File, offset int64) (uint32, error) {
	if _, err := f.Seek(offset, io.SeekStart); err != nil {
		return 0, err
	}
	buf := make([]byte, 4)
	if _, err := io.ReadFull(f, buf); err != nil {
		return 0, err
	}
	return binary.BigEndian.Uint32(buf), nil
}

func expectMagic(f *os.File, offset int64, want string) error {
	if _, err := f.Seek(offset, io.SeekStart); err != nil {
		return err
	}
	buf := make([]byte, len(want))
	if _, err := io.ReadFull(f, buf); err != nil {
		return err
	}
	if got := string(buf); got != want {
		return fmt.Errorf("bad magic: got %q, want %q", got, want)
	}
	return nil
}

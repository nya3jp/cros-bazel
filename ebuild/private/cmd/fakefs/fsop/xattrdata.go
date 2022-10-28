// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package fsop

import (
	"fmt"
	"strconv"
	"strings"
)

type overrideData struct {
	Uid int
	Gid int
}

var defaultOverrideData = &overrideData{
	Uid: 0,
	Gid: 0,
}

func parseOverrideData(b []byte) (*overrideData, error) {
	v := strings.Split(string(b), ":")
	if len(v) != 2 {
		return nil, fmt.Errorf("corrupted override data: %s", string(b))
	}
	uid, err := strconv.Atoi(v[0])
	if err != nil {
		return nil, fmt.Errorf("corrupted override data: corrupted uid: %s", v[0])
	}
	gid, err := strconv.Atoi(v[1])
	if err != nil {
		return nil, fmt.Errorf("corrupted override data: corrupted gid: %s", v[1])
	}
	return &overrideData{
		Uid: uid,
		Gid: gid,
	}, nil
}

func (o *overrideData) Marshal() []byte {
	return []byte(fmt.Sprintf("%d:%d", o.Uid, o.Gid))
}

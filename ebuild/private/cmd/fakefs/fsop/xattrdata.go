// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package fsop

import (
	"fmt"
	"strconv"
	"strings"
)

type ownershipData struct {
	Uid int
	Gid int
}

var defaultOwnershipData = &ownershipData{
	Uid: 0,
	Gid: 0,
}

func parseOwnershipData(b []byte) (*ownershipData, error) {
	v := strings.Split(string(b), ":")
	if len(v) != 2 {
		return nil, fmt.Errorf("corrupted ownership data: %s", string(b))
	}
	uid, err := strconv.Atoi(v[0])
	if err != nil {
		return nil, fmt.Errorf("corrupted ownership data: corrupted uid: %s", v[0])
	}
	gid, err := strconv.Atoi(v[1])
	if err != nil {
		return nil, fmt.Errorf("corrupted ownership data: corrupted gid: %s", v[1])
	}
	return &ownershipData{
		Uid: uid,
		Gid: gid,
	}, nil
}

func (o *ownershipData) Marshal() []byte {
	return []byte(fmt.Sprintf("%d:%d", o.Uid, o.Gid))
}

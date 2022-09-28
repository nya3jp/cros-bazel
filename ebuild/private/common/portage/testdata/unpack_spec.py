#!/usr/bin/env python3
# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Unpacks a spec file (a plain text containing multiple files).
"""

import argparse
import os
import shutil


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('spec_file')
    parser.add_argument('out_dir')
    options = parser.parse_args()

    shutil.rmtree(options.out_dir, ignore_errors=True)
    os.makedirs(options.out_dir, exist_ok=True)
    current_out = None

    with open(options.spec_file) as spec_in:
        for line in spec_in:
            if line.startswith('>>>'):
                if current_out:
                    current_out.close()
                    current_out = None
                filename = line[3:].strip()
                target = None
                if ' -> ' in filename:
                    filename, target = filename.split(' -> ', 2)
                path = os.path.join(options.out_dir, filename)
                os.makedirs(os.path.dirname(path), exist_ok=True)
                if target:
                    os.symlink(target, path)
                else:
                    current_out = open(path, 'w')
            elif current_out:
                current_out.write(line)

    if current_out:
        current_out.close()


if __name__ == '__main__':
    main()

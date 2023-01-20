#!/bin/bash

set -eu -o pipefail

GSUTIL="$1"
SRC="$2"
DST="$3"

protocol="${SRC%%://*}"

if [[ "${protocol}" = http ]] || [[ "${protocol}" = https ]]; then
  wget "${SRC}" -O "${DST}"
elif [[ "${protocol}" = "gs" ]]; then
  "${GSUTIL}" cp "${SRC}" "${DST}"
else
  cp "${SRC}" "${DST}"
fi
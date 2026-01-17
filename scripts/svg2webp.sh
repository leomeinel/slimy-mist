#!/usr/bin/env bash
###
# File: svg2webp.sh
# Author: Leopold Johannes Meinel (leo@meinel.dev)
# -----
# Copyright (c) 2026 Leopold Johannes Meinel & contributors
# SPDX ID: Apache-2.0
# URL: https://www.apache.org/licenses/LICENSE-2.0
###

# Fail on error
set -e

# Set ${SCRIPT_DIR}
SCRIPT_DIR="$(dirname -- "$(readlink -f -- "${0}")")"

# Create webp with specified dimensions from svg
tmpfile="$(mktemp /tmp/"$(basename "${0}")"-XXXXXX.png)"
for file in "${SCRIPT_DIR}"/svg2webp/*.svg; do
    read -rp "Width to use for '$(basename "${file}")': " WIDTH
    read -rp "Height to use for '$(basename "${file}")': " HEIGHT
    OUTPUT="${file%.*}.webp"
    svgo "${file}"
    inkscape "${file}" -w "${WIDTH}" -h "${HEIGHT}" -o "${tmpfile}"
    magick "${tmpfile}" -background none "${OUTPUT}"
done

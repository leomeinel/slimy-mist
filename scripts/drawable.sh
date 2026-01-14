#!/usr/bin/env bash
###
# File: drawable.sh
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

# Activate venv
[[ -d "${SCRIPT_DIR}"/.venv-pixels2svg ]] ||
    python -m venv "${SCRIPT_DIR}"/.venv-pixels2svg
. "${SCRIPT_DIR}"/.venv-pixels2svg/bin/activate
pip install pixels2svg

# Create android vector drawable from webp scaled for use as icon
tmpfile="$(mktemp /tmp/"$(basename "${0}")"-XXXXXX)"
for file in "${SCRIPT_DIR}"/drawables/*.webp; do
    magick "${file}" -filter point -resize 66x66 -background none -gravity center -extent 108x108 "${tmpfile}"
    SVG_OUTPUT="${file%.*}.svg"
    python -m pixels2svg --no_pretty "${tmpfile}" >"${SVG_OUTPUT}"
    svgo "${SVG_OUTPUT}"
    OUTPUT="${SVG_OUTPUT%.*}.xml"
    npx s2v -i "${SVG_OUTPUT}" -o "${OUTPUT}"
done

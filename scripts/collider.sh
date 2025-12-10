#!/usr/bin/env bash
###
# File: collider.sh
# Author: Leopold Johannes Meinel (leo@meinel.dev)
# -----
# Copyright (c) 2025 Leopold Johannes Meinel & contributors
# SPDX ID: Apache-2.0
# URL: https://www.apache.org/licenses/LICENSE-2.0
###

# Fail on error
set -e

# Set ${SCRIPT_DIR}
SCRIPT_DIR="$(dirname -- "$(readlink -f -- "${0}")")"

for file in "${SCRIPT_DIR}"/colliders/*.webp; do
    OUTPUT="${file%.*}.collision.ron"
    FULL_W="$(magick "${file}" -format "%w" info:)"
    FULL_H="$(magick "${file}" -format "%h" info:)"
    X_OFF="$(magick "${file}" -trim -format "%X" info:)"
    Y_OFF="$(magick "${file}" -trim -format "%Y" info:)"
    TRIM_W="$(magick "${file}" -trim -format "%w" info:)"
    TRIM_H="$(magick "${file}" -trim -format "%h" info:)"

    POINTS="$(magick -size "${FULL_W}x${FULL_H}" xc:white -fill none -stroke black -strokewidth 1 -draw "rectangle ${X_OFF},${Y_OFF} $((X_OFF + TRIM_W -1)),$((Y_OFF + TRIM_H -1))" txt:- | grep "black" | cut -d: -f1)"

    # shellcheck disable=SC2001
    {
        printf '%s\n' "CollisionData ("
        printf '%s\n' "    vertices: ["
        printf '%s\n' "${POINTS//,/., }" | sed 's/^/        (/' | sed 's/$/.),/'
        printf '%s\n' "    ],"
        printf '%s\n' ")"
    } >"${OUTPUT}"
done

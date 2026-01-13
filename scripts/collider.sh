#!/usr/bin/env bash
###
# File: collider.sh
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

for file in "${SCRIPT_DIR}"/colliders/*.webp; do
    OUTPUT="${file%.*}.collision.ron"
    TRIM_W="$(magick "${file}" -trim -format "%w" info:)"
    TRIM_H="$(magick "${file}" -trim -format "%h" info:)"

    printf '%s\n' "Valid shapes are:"
    printf '%s\n' "- ball"
    printf '%s\n' "- capsule_x"
    printf '%s\n' "- capsule_y"
    printf '%s\n' "- cuboid"
    read -rp "Shape to use for '$(basename "${file}")': " SHAPE

    {
        printf '%s\n' "CollisionData ("
        printf '%s\n' "    shape: Some(\"${SHAPE}\"),"
        printf '%s\n' "    width: Some(${TRIM_W}),"
        printf '%s\n' "    height: Some(${TRIM_H}),"
        printf '%s\n' ")"
    } >"${OUTPUT}"
done

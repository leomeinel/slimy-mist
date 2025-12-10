#!/usr/bin/env bash
###
# File: build.sh
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

# Build specific build for given argument
if [[ -z "${1}" ]]; then
    cargo build --no-default-features --release
elif [[ "${1}" == "web" ]]; then
    rustup target add wasm32-unknown-unknown
    cargo build --no-default-features --target wasm32-unknown-unknown --profile web-release
    ## Optimize binary
    PACKAGE_NAME="$(tomlq -r '.package.name' "${SCRIPT_DIR}"/Cargo.toml)"
    OUTPUT="${SCRIPT_DIR}"/target/wasm32-unknown-unknown/web-release/"${PACKAGE_NAME}".wasm
    tmpfile="$(mktemp /tmp/"${PACKAGE_NAME}"-XXXXXX.wasm)"
    mv "${OUTPUT}" "${tmpfile}"
    wasm-opt -Os -o "${OUTPUT}" "${tmpfile}" --enable-bulk-memory-opt --enable-nontrapping-float-to-int
    rm -f "${tmpfile}"
elif [[ "${1}" == "web-dev" ]]; then
    rustup target add wasm32-unknown-unknown
    cargo build --target wasm32-unknown-unknown --profile web-dev
else
    cargo build
fi

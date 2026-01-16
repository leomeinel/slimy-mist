#!/usr/bin/env bash
###
# File: run.sh
# Author: Leopold Johannes Meinel (leo@meinel.dev)
# -----
# Copyright (c) 2026 Leopold Johannes Meinel & contributors
# SPDX ID: Apache-2.0
# URL: https://www.apache.org/licenses/LICENSE-2.0
###

# Fail on error
set -e

# Set env variables
SCRIPT_DIR="$(dirname -- "$(readlink -f -- "${0}")")"
BINARY_NAME="slimymist"

# Define functions
run_web() {
    PACKAGE_NAME="$(tomlq -r '.package.name' "${SCRIPT_DIR}"/../Cargo.toml)"
    OUTPUT="${SCRIPT_DIR}"/../target/wasm32-unknown-unknown/"${1}"/"${PACKAGE_NAME}".wasm
    CARGO_XDG_DIR=~/.local/share/cargo/bin
    CARGO_DIR=~/.cargo/bin
    WASM_RUNNER="wasm-server-runner"
    if command -v "${WASM_RUNNER}" >/dev/null 2>&1; then
        "${WASM_RUNNER}" "${OUTPUT}"
    elif [[ -f "${CARGO_XDG_DIR}"/"${WASM_RUNNER}" ]]; then
        "${CARGO_XDG_DIR}"/"${WASM_RUNNER}" "${OUTPUT}"
    elif [[ -f "${CARGO_DIR}"/"${WASM_RUNNER}" ]]; then
        "${CARGO_DIR}"/"${WASM_RUNNER}" "${OUTPUT}"
    else
        printf '%s\n' "ERROR: ${WASM_RUNNER} not found"
    fi
}

# Run specific build for given argument
if [[ -z "${1}" ]]; then
    cargo run --bin "${BINARY_NAME}" --no-default-features --release
elif [[ "${1}" == "web" ]]; then
    "${SCRIPT_DIR}"/build.sh "${1}"
    run_web "${1}"
elif [[ "${1}" == "web-dev" ]]; then
    "${SCRIPT_DIR}"/build.sh "${1}"
    run_web "${1}"
else
    if command -v mangohud >/dev/null 2>&1; then
        mangohud cargo run --bin "${BINARY_NAME}"
    else
        cargo run --bin "${BINARY_NAME}"
    fi
fi

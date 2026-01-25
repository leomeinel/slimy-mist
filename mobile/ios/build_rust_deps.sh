#!/usr/bin/env bash
###
# File: build_rust_deps.sh
# Author: Leopold Johannes Meinel (leo@meinel.dev)
# -----
# Copyright (c) 2026 Leopold Johannes Meinel & contributors
# SPDX ID: Apache-2.0
# URL: https://www.apache.org/licenses/LICENSE-2.0
# -----
# Heavily inspired by:
# - https://github.com/mozilla/glean/blob/main/build-scripts/xc-universal-binary.sh
# - https://github.com/bevyengine/bevy/tree/main/examples/mobile
###

# FIXME: Make this independent of xcode?

set -eux

PATH=${PATH}:${HOME}/.cargo/bin

# Set ${SCRIPT_DIR}
SCRIPT_DIR="$(dirname -- "$(readlink -f -- "${0}")")"
cd "${SCRIPT_DIR}"/../..

set -euvx

# add homebrew bin path, as it's the most commonly used package manager on macOS
# this is needed for cmake on apple arm processors as it's not available by default
export PATH="${PATH}:/opt/homebrew/bin"

# Make Cargo output cache files in Xcode's directories
export CARGO_TARGET_DIR="${DERIVED_FILE_DIR}/cargo"

# Xcode places `/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin`
# at the front of the path, with makes the build fail with `ld: library 'System' not found`, upstream issue:
# <https://github.com/rust-lang/rust/issues/80817>.
#
# Work around it by resetting the path, so that we use the system `cc`.
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:${PATH}"

IS_SIMULATOR=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
    IS_SIMULATOR=1
fi

EXECUTABLES=()
for arch in ${ARCHS}; do
    case "${arch}" in
    x86_64)
        if [ ${IS_SIMULATOR} -eq 0 ]; then
            echo "Building for x86_64, but not a simulator build. What's going on?" >&2
            exit 2
        fi

        # Intel iOS simulator
        export CFLAGS_x86_64_apple_ios="-target x86_64-apple-ios"
        TARGET=x86_64-apple-ios
        ;;

    arm64)
        if [ ${IS_SIMULATOR} -eq 0 ]; then
            # Hardware iOS targets
            TARGET=aarch64-apple-ios
        else
            # M1 iOS simulator
            TARGET=aarch64-apple-ios-sim
        fi
        ;;
    esac

    cargo build --bin slimymist --no-default-features --release --locked --target="${TARGET}" -j default
    # Collect the executables
    EXECUTABLES+=("${DERIVED_FILE_DIR}/cargo/${TARGET}/release/slimymist")
done

# Combine executables, and place them at the output path excepted by Xcode
lipo -create -output "${TARGET}_BUILD_DIR/${EXECUTABLE_PATH}" "${EXECUTABLES[@]}"

#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST=pi@raspotify
readonly TARGET_PATH=/home/pi/rusty_volume
readonly TARGET_ARCH=aarch64-unknown-linux-gnu
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/rusty_volume
readonly GLIBC_VERSION=2.36

cargo zigbuild --release --target=${TARGET_ARCH}.${GLIBC_VERSION} --verbose
#cargo build --release --target=${TARGET_ARCH} --verbose

rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} ${TARGET_PATH}
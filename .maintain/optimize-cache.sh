#!/bin/sh
#

set -x

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../

cd ${WORK_PATH}

cargo clean -p pangolin-runtime --release
cargo clean -p pangoro-runtime --release
cargo clean -p librocksdb-sys --release
cargo clean -p drml-service --release


rm -rf target/release/wbuild/pangolin-runtime/
rm -rf target/release/wbuild/pangoro-runtime/
rm -rf target/srtool
rm -rf target/subwasm

#!/bin/sh
#

set -xe

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../

cd ${WORK_PATH}

cargo clean -p crab-runtime --release
cargo clean -p darwinia-runtime --release
cargo clean -p librocksdb-sys --release
cargo clean -p darwinia-node-service --release
cargo clean -p darwinia-cli --release


rm -rf target/release/wbuild/crab-runtime/
rm -rf target/release/wbuild/darwinia-runtime/

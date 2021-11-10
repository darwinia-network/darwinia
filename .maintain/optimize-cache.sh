#!/bin/sh
#

set -x

cargo clean -p pangolin-runtime --release
cargo clean -p pangoro-runtime --release
cargo clean -p librocksdb-sys --release
cargo clean -p drml-service --release


rm -rf target/release/wbuild/pangolin-runtime/
rm -rf target/release/wbuild/pangoro-runtime/
rm -rf target/srtool
rm -rf target/subwasm

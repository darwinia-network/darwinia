#!/bin/sh

cargo clean -p darwinia 2> /dev/null || true
cargo clean -p darwinia-service 2> /dev/null || true
cargo clean -p darwinia-runtime 2> /dev/null || true
cargo clean -p crab-runtime 2> /dev/null || true
cargo clean -p darwinia-cli 2> /dev/null || true
cargo clean -p librocksdb-sys 2> /dev/null || true
rm -rf target/debug/wbuild 2> /dev/null || true

#!/bin/sh

cargo clean --release -p darwinia 2> /dev/null || true
cargo clean --release -p darwinia-runtime 2> /dev/null || true
rm -rf target/release/wbuild 2> /dev/null || true

cargo clean -p darwinia 2> /dev/null || true
cargo clean -p darwinia-runtime 2> /dev/null || true
rm -rf target/debug/wbuild 2> /dev/null || true

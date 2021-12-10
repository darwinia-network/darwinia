#!/bin/sh

cargo clean --release -p darwinia 2> /dev/null || true
cargo clean --release -p darwinia-service 2> /dev/null || true
cargo clean --release -p darwinia-runtime 2> /dev/null || true
cargo clean --release -p crab-runtime 2> /dev/null || true
rm -rf target/release/wbuild 2> /dev/null || true

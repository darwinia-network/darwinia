#!/bin/sh
#

rustup default ${RUST_TOOLCHAIN}
rustup target add wasm32-unknown-unknown

mkdir -p /build/target/srtool

build --json --save

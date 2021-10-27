#!/bin/sh
#

set -xe

echo ${RUSTC_VERSION}

rustup default ${RUSTC_VERSION}
rustup target add wasm32-unknown-unknown

mkdir -p /build/target/srtool

build --json --save

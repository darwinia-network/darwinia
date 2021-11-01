#!/bin/sh
#

set -x

echo ${RUSTC_VERSION}

rustup default ${RUSTC_VERSION}
rustup target add wasm32-unknown-unknown

rm -rf /build/target/srtool
mkdir -p /build/target/srtool

build --json --save

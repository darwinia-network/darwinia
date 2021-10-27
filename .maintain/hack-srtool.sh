#!/bin/sh
#

rustup default $1
rustup target add wasm32-unknown-unknown

mkdir -p /build/target/srtool

build --json --save

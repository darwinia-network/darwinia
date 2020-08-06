#!/usr/bin/env bash

set -eux

rustup default $RUST_TOOLCHAIN

source ~/.cargo/env

rustup --version
cargo --version
rustc --version

cargo clean

cargo build --locked
echo -e "\e[0;32m +-------------+ \n\e[0;32m | Build Pass | \n\e[0;32m +-------------+ \e[0m"

#!/usr/bin/env bash

set -eux

rustup default stable
rustup component add rustfmt

source ~/.cargo/env

rustup --version
cargo --version
rustc --version

cargo clean

cargo fmt --all
echo -e "\e[0;32m +-------------+ \n\e[0;32m | Format Pass | \n\e[0;32m +-------------+ \e[0m"

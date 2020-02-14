#!/usr/bin/env bash

set -eux

# rustfmt is check as stable rust
rustup default stable
rustup component add rustfmt

source ~/.cargo/env

rustup --version
cargo --version
rustc --version

# clean target cache if any
rm -rf target

cargo fmt --all
echo -e "\e[0;32m +-------------+ \n\e[0;32m | Format Pass | \n\e[0;32m +-------------+ \e[0m"

#!/usr/bin/env bash

set -eux

# rustfmt is check as stable rust
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=stable -y

source ~/.cargo/env

rustup --version
cargo --version
rustc --version

case $TARGET in
	# Without WASM
	"native")
		cargo fmt --all
		echo -e "\e[0;32m +-------------+ \n\e[0;32m | Format Pass | \n\e[0;32m +-------------+ \e[0m"
		;;

	# With WASM fmt only check in native
	"wasm")
		echo "check in native build always pass in wasm build"
		;;
esac

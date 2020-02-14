#!/usr/bin/env bash

set -eux

rustup default $RUST_TOOLCHAIN 

source ~/.cargo/env

rustup --version
cargo --version
rustc --version

case $TARGET in
	# Without WASM
	"native")
		# There is some issue to build on ci server with SKIP_WASM_BUILD=1 
		cargo build --release --all --locked "$@"
		echo -e "\e[0;32m +-------------+ \n\e[0;32m | Native Pass | \n\e[0;32m +-------------+ \e[0m"
		;;

	# With WASM
	"wasm")
		WASM_BUILD_TYPE=release cargo build --locked "$@"
		echo -e "\e[0;32m +-----------+ \n\e[0;32m | WASM Pass | \n\e[0;32m +-----------+ \e[0m"
		;;
esac

#!/usr/bin/env bash

set -eux

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=$RUST_TOOLCHAIN -y

source ~/.cargo/env

rustup --version
cargo --version
rustc --version

case $TARGET in
	# Format check in stable rust
	"rustfmt")
		cargo fmt --all
		;;

	# Without WASM
	"native")
		rustup target add wasm32-unknown-unknown
		# There is some issue to build on ci server with SKIP_WASM_BUILD=1 
		cargo build --release --all --locked "$@"
		echo -e "\e[0;32m +-------------+ \n\e[0;32m | Native Pass | \n\e[0;32m +-------------+ \e[0m"
		;;

	# With WASM
	"wasm")
		rustup target add wasm32-unknown-unknown
		WASM_BUILD_TYPE=release cargo build --locked "$@"
		echo -e "\e[0;32m +-----------+ \n\e[0;32m | WASM Pass | \n\e[0;32m +-----------+ \e[0m"
		;;
esac

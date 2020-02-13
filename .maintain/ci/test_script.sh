#!/usr/bin/env bash

set -eux

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=$RUST_TOOLCHAIN -y

source ~/.cargo/env

rustup --version
cargo --version
rustc --version

case $TARGET in
	# Without WASM, build then test
	"native")
		rustup target add wasm32-unknown-unknown
		# There is some issue to build on ci server with SKIP_WASM_BUILD=1 
		cargo test --release --all --locked "$@"
		echo -e "\e[0;32m +------------+ \n\e[0;32m | Release OK | \n\e[0;32m +------------+ \e[0m"
		;;

	# With WASM, build then test
	"wasm")
		WASM_BUILD_TYPE=release cargo test --locked "$@"
		echo -e "\e[0;32m +------------+ \n\e[0;32m | Release OK | \n\e[0;32m +------------+ \e[0m"
		;;
esac

#!/usr/bin/env bash
echo -e "Test Darwinia ${1} ..."

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
		TARGET=native cargo test -p darwinia-${1}
		echo -e "\e[0;32m +------------+ \n\e[0;32m  | ${1}  Pass | \n\e[0;32m  +------------+ \e[0m"
		;;

	# With WASM
	"wasm")
		WASM_BUILD_TYPE=release cargo test -p darwinia-${1}
		echo -e "\e[0;32m +------------+ \n\e[0;32m  | ${1}  Pass | \n\e[0;32m  +------------+ \e[0m"
		;;
esac

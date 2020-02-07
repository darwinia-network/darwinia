#!/usr/bin/env bash

set -eux

source ~/.cargo/env

case $TARGET in
	# Format check in stable rust
	"rustfmt")
		echo -e "\e[0;32m +------------+ \n | No Test    | \n +------------+ \e[0m"
		;;

	# Without WASM
	"native")
		rustup target add wasm32-unknown-unknown
		SKIP_WASM_BUILD=1 cargo test -p darwinia-${1}
		echo -e "\e[0;32m +------------+ \n\e[0;32m  | ${1}  Pass | \n\e[0;32m  +------------+ \e[0m"
		;;

	# With WASM
	"wasm")
		WASM_BUILD_TYPE=release cargo test -p darwinia-kton "$@"
		echo -e "\e[0;32m +------------+ \n | Kton  Pass | \n +------------+ \e[0m"
		WASM_BUILD_TYPE=release cargo test -p darwinia-ring "$@"
		echo -e "\e[0;32m +------------+ \n | Ring  Pass | \n +------------+ \e[0m"
		WASM_BUILD_TYPE=release cargo test -p darwinia-staking "$@"
		echo -e "\e[0;32m +------------+ \n | Staking OK | \n +------------+ \e[0m"
		;;
esac

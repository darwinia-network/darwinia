#!/usr/bin/env bash

set -eux

source ~/.cargo/env

case $TARGET in
	# Format check in stable rust
	"rustfmt")
		echo -e "\e[0;32m +------------+ \n\e[0;32m | No Test    | \n\e[0;32m +------------+ \e[0m"
		;;

	# Without WASM, build then test
	"native")
		SKIP_WASM_BUILD=1 cargo test --release --all --locked "$@"
		echo -e "\e[0;32m +------------+ \n\e[0;32m | Release OK | \n\e[0;32m +------------+ \e[0m"
		;;

	# With WASM, build then test
	"wasm")
		WASM_BUILD_TYPE=release cargo test --locked "$@"
		echo -e "\e[0;32m +------------+ \n\e[0;32m | Release OK | \n\e[0;32m +------------+ \e[0m"
		;;
esac

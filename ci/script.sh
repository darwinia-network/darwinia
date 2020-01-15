#!/usr/bin/env bash

set -eux

# Install rustup and the specified rust toolchain.
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=$RUST_TOOLCHAIN -y

# Load cargo environment. Specifically, put cargo into PATH.
source ~/.cargo/env

# Install wasm toolchain
rustup target add wasm32-unknown-unknown

rustup --version
cargo --version
rustc --version

case $TARGET in
	"rustfmt")
		sudo apt-get -y update
		sudo apt-get install -y cmake pkg-config libssl-dev
		cargo fmt --all
		;;

	"native")
		# Unit test
		cargo test --release --all --locked "$@"
		;;

	"wasm")
		# Build test
		cargo build --locked "$@"
		;;
esac

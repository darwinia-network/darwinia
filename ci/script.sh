#!/usr/bin/env bash

set -eux

# Install rustup and the specified rust toolchain.
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=$1 -y

# Load cargo environment. Specifically, put cargo into PATH.
source ~/.cargo/env

# Make sure using the nightly toolchain
rustup default nightly

# Install wasm toolchain
rustup target add wasm32-unknown-unknown

rustup --version
cargo --version
rustc --version

case $2 in
	"native")
		sudo apt-get -y update
		sudo apt-get install -y cmake pkg-config libssl-dev

		# Unit test
		cargo test --release --all --locked
		;;

	"wasm")
		# Build test
		cargo build
		;;
esac
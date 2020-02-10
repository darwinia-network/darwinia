#!/usr/bin/env bash

set -eux

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=$RUST_TOOLCHAIN -y

source ~/.cargo/env

rustup --version
cargo --version
rustc --version

case $TARGET in
	# Format check
	"rustfmt")
		rustup component add rustfmt-preview
		cargo fmt --all
		;;

	# Unit test
	"native")
		cargo test --release --all --locked "$@"
		;;

	# Build test
	"wasm")
		rustup target add wasm32-unknown-unknown
		cargo build --locked "$@"
		;;
esac

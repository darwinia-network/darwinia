#!/usr/bin/env bash

# The script help you set up your develop envirnment

# Setup git hooks
cp .hooks/* .git/hooks

# Install nightly Rust
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=nightly -y

# Install wasm toolchain
rustup target add wasm32-unknown-unknown

# Install rustfmt for coding style checking
rustup component add rustfmt --toolchain nightly

# TODO: help other developers with different platform
sudo apt-get -y update
sudo apt-get install -y cmake pkg-config libssl-dev

#!/bin/sh
#
#


set -ex

BIN_PATH=$(dirname $(readlink -f $0))

#####
## warning: "`background_threads_runtime_support` not supported for `x86_64-unknown-linux-musl`"
## error: failed to run custom build command for `jemalloc-sys v0.3.2`
#####

#apk update
#apk add clang clang-dev cmake musl-dev

rustup default ${RUST_TOOLCHAIN}

if [ "$ARCH" = "x86_64" ]; then
  apt-get update
  apt-get install -y --no-install-recommends clang libclang-dev cmake

  rustup target add x86_64-unknown-linux-gnu
  rustup target add wasm32-unknown-unknown
fi
if [ "$ARCH" = "aarch64" ]; then
  echo 'aarch64'
fi



cd /data/darwinia

if [ -n "${OVERALL_TEST}" ]; then

  cargo test

else

  if [ "$ARCH" = "aarch64" ]; then
    SKIP_WASM_BUILD=1 cross build --no-default-features --locked --release --target aarch64-unknown-linux-gnu
  else
    cargo build --release
  fi

fi



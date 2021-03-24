#!/bin/sh
#
#
#


BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../

docker run --rm -i \
  -v ${WORK_PATH}:/data/darwinia \
  -e RUST_TOOLCHAIN=${RUST_TOOLCHAIN} \
  -e CARGO_TERM_COLOR=always \
  -e OVERALL_TEST=true \
  -e ARCH=x86_64 \
  rust:1 \
  sh -f /data/darwinia/.maintain/docker/_build-darwinia.sh



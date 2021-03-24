#!/bin/sh
#
#
#

set -x

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../


BRANCH_NAME=$(echo $GITHUB_REF | cut -d'/' -f 3)

docker run --rm -i \
  -e PACKAGE=darwinia-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -v ${WORK_PATH}:/build \
  paritytech/srtool:${RUST_TOOLCHAIN} || exit 1


docker run --rm -i \
  -e PACKAGE=crab-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -v ${WORK_PATH}:/build \
  paritytech/srtool:${RUST_TOOLCHAIN} || exit 1


#tree target/srtool


mkdir -p ${WORK_PATH}/bin

cp ${WORK_PATH}/target/srtool/release/wbuild/darwinia-runtime/darwinia_runtime.compact.wasm ${WORK_PATH}/bin/
cp ${WORK_PATH}/target/srtool/release/wbuild/crab-runtime/crab_runtime.compact.wasm ${WORK_PATH}/bin/

cp ${WORK_PATH}/target/srtool/release/wbuild/target/wasm32-unknown-unknown/release/darwinia_runtime.wasm ${WORK_PATH}/bin/
cp ${WORK_PATH}/target/srtool/release/wbuild/target/wasm32-unknown-unknown/release/crab_runtime.wasm ${WORK_PATH}/bin/

ls ${WORK_PATH}/bin/

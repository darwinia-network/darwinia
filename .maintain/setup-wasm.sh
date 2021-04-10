#!/bin/sh
#
#
#

set -xe

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../


BRANCH_NAME=$(echo $GITHUB_REF | cut -d'/' -f 3)

## notice:
## `"-e BUILD_OPTS=--manifest-path /build/Cargo.toml"` is hack,
## currently the darwinia don't support `--features on-chain-release-build` to disable runtime logger,
## because the substrate latest version has not yet been merged,
## when after do that and add `on-chain-release-build` features, this line can be remove.

## 1. chevdor/srtool:nightly-2021-02-25 have a bug, --json and --app upside down, use other version need remove it.
EXTRA_ARGS='--json'

docker run --rm -i \
  -e PACKAGE=darwinia-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -e BUILD_OPTS="--manifest-path /build/Cargo.toml" \
  -v ${WORK_PATH}:/build \
  chevdor/srtool:${RUST_TOOLCHAIN} build ${EXTRA_ARGS} \
  | tee ${WORK_PATH}/build-darwinia-wasm.log


docker run --rm -i \
  -e PACKAGE=crab-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -e BUILD_OPTS="--manifest-path /build/Cargo.toml" \
  -v ${WORK_PATH}:/build \
  chevdor/srtool:${RUST_TOOLCHAIN} build ${EXTRA_ARGS} \
  | tee ${WORK_PATH}/build-crab-wasm.log


mkdir -p ${WORK_PATH}/deploy/bin

_PROPOSAL_DARWINIA=$(cat ${WORK_PATH}/build-darwinia-wasm.log | grep 'Proposal\s\+:')
_PROPOSAL_CRAB=$(cat ${WORK_PATH}/build-crab-wasm.log | grep 'Proposal\s\+:')

PROPOSAL_DARWINIA=0x${_PROPOSAL_DARWINIA#*0x}
PROPOSAL_CRAB=0x${_PROPOSAL_CRAB#*0x}

PROPOSAL_DARWINIA=$(echo ${PROPOSAL_DARWINIA} | sed 's/[^[:print:]]\[0m//g')
PROPOSAL_CRAB=$(echo ${PROPOSAL_CRAB} | sed 's/[^[:print:]]\[0m//g')


echo ${PROPOSAL_DARWINIA} > ${WORK_PATH}/deploy/bin/${PROPOSAL_DARWINIA}.proposal.darwinia.txt
echo ${PROPOSAL_CRAB} > ${WORK_PATH}/deploy/bin/${PROPOSAL_CRAB}.proposal.crab.txt

### before srtool 2021-03-15 use this
cp ${WORK_PATH}/target/srtool/release/wbuild/darwinia-runtime/darwinia_runtime.compact.wasm \
  ${WORK_PATH}/deploy/bin/
cp ${WORK_PATH}/target/srtool/release/wbuild/crab-runtime/crab_runtime.compact.wasm \
  ${WORK_PATH}/deploy/bin/

cp ${WORK_PATH}/target/srtool/release/wbuild/darwinia-runtime/target/wasm32-unknown-unknown/release/darwinia_runtime.wasm \
  ${WORK_PATH}/deploy/bin/
cp ${WORK_PATH}/target/srtool/release/wbuild/crab-runtime/target/wasm32-unknown-unknown/release/crab_runtime.wasm \
  ${WORK_PATH}/deploy/bin/

### after srtool 2021-03-15 use this
#cp ${WORK_PATH}/runtime/darwinia/target/srtool/release/wbuild/darwinia-runtime/darwinia_runtime.compact.wasm \
#  ${WORK_PATH}/deploy/bin/
#cp ${WORK_PATH}/runtime/crab/target/srtool/release/wbuild/crab-runtime/crab_runtime.compact.wasm \
#  ${WORK_PATH}/deploy/bin/

ls ${WORK_PATH}/deploy/bin/

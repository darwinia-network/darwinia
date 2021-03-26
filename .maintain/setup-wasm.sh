#!/bin/sh
#
#
#

set -xe

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../


BRANCH_NAME=$(echo $GITHUB_REF | cut -d'/' -f 3)

docker run --rm -i \
  -e PACKAGE=darwinia-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -v ${WORK_PATH}:/build \
  paritytech/srtool:${RUST_TOOLCHAIN} | tee ${WORK_PATH}/build-darwinia-wasm.log


docker run --rm -i \
  -e PACKAGE=crab-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -v ${WORK_PATH}:/build \
  paritytech/srtool:${RUST_TOOLCHAIN} | tee ${WORK_PATH}/build-crab-wasm.log


mkdir -p ${WORK_PATH}/bin

_PROPOSAL_DARWINIA=$(grep 'Proposal :' ${WORK_PATH}/build-darwinia-wasm.log)
_PROPOSAL_CRAB=$(grep 'Proposal :' ${WORK_PATH}/build-crab-wasm.log)

PROPOSAL_DARWINIA=0x${PROPOSAL_DARWINIA#*0x}
PROPOSAL_CRAB=0x${PROPOSAL_CRAB#*0x}

echo ${PROPOSAL_DARWINIA} > ${WORK_PATH}/bin/${PROPOSAL_DARWINIA}.proposal.darwinia.txt
echo ${PROPOSAL_CRAB} > ${WORK_PATH}/bin/${PROPOSAL_CRAB}.proposal.crab.txt

cp ${WORK_PATH}/target/srtool/release/wbuild/darwinia-runtime/darwinia_runtime.compact.wasm ${WORK_PATH}/bin/
cp ${WORK_PATH}/target/srtool/release/wbuild/crab-runtime/crab_runtime.compact.wasm ${WORK_PATH}/bin/

cp ${WORK_PATH}/target/srtool/release/wbuild/target/wasm32-unknown-unknown/release/darwinia_runtime.wasm ${WORK_PATH}/bin/
cp ${WORK_PATH}/target/srtool/release/wbuild/target/wasm32-unknown-unknown/release/crab_runtime.wasm ${WORK_PATH}/bin/

ls ${WORK_PATH}/bin/

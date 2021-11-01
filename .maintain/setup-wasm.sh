#!/bin/sh
#
#
#

set -xe

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../


BRANCH_NAME=$(echo $GITHUB_REF | cut -d'/' -f 3)

mkdir -p ${WORK_PATH}/deploy/bin

PATH_REPORT=${WORK_PATH}/target/srtool/srtool-wasm-report*.txt

# doc: https://github.com/darwinia-network/darwinia/pull/764#discussion_r737966278

# darwinia

docker run --rm -i \
  -e PACKAGE=darwinia-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -e RUSTC_VERSION=${RUST_TOOLCHAIN} \
  -v ${WORK_PATH}:/build \
  --entrypoint bash \
  chevdor/srtool:1.53.0 /build/.maintain/hack-srtool.sh

_PROPOSAL_DARWINIA_COMPACT=$(cat ${PATH_REPORT} | jq -r .runtimes.compact.prop)
_PROPOSAL_DARWINIA_COMPRESSED=$(cat ${PATH_REPORT} | jq -r .runtimes.compressed.prop)
_WASM_DARWINIA_COMPACT=$(cat ${PATH_REPORT} | jq -r .runtimes.compact.wasm)
_WASM_DARWINIA_COMPRESSED=$(cat ${PATH_REPORT} | jq -r .runtimes.compressed.wasm)

PATH_PROPOSAL_DARWINIA=${WORK_PATH}/deploy/bin/proposal.darwinia.txt
echo "compact: ${_PROPOSAL_DARWINIA_COMPACT}" >> ${PATH_PROPOSAL_DARWINIA}
echo "compressed: ${_PROPOSAL_DARWINIA_COMPRESSED}" >> ${PATH_PROPOSAL_DARWINIA}
echo '' >> ${PATH_PROPOSAL_DARWINIA}

cp ${WORK_PATH}/${_WASM_DARWINIA_COMPACT} ${WORK_PATH}/deploy/bin/
cp ${WORK_PATH}/${_WASM_DARWINIA_COMPRESSED} ${WORK_PATH}/deploy/bin/


# crab

docker run --rm -i \
  -e PACKAGE=crab-runtime \
  -e VERBOSE=1 \
  -e CARGO_TERM_COLOR=always \
  -e RUSTC_VERSION=${RUST_TOOLCHAIN} \
  -v ${WORK_PATH}:/build \
  --entrypoint bash \
  chevdor/srtool:1.53.0 /build/.maintain/hack-srtool.sh

_PROPOSAL_CRAB_COMPACT=$(cat ${PATH_REPORT} | jq -r .runtimes.compact.prop)
_PROPOSAL_CRAB_COMPRESSED=$(cat ${PATH_REPORT} | jq -r .runtimes.compressed.prop)
_WASM_CRAB_COMPACT=$(cat ${PATH_REPORT} | jq -r .runtimes.compact.wasm)
_WASM_CRAB_COMPRESSED=$(cat ${PATH_REPORT} | jq -r .runtimes.compressed.wasm)

PATH_PROPOSAL_CRAB=${WORK_PATH}/deploy/bin/proposal.crab.txt
echo "compact: ${_PROPOSAL_CRAB_COMPACT}" >> ${PATH_PROPOSAL_CRAB}
echo "compressed: ${_PROPOSAL_CRAB_COMPRESSED}" >> ${PATH_PROPOSAL_CRAB}
echo '' >> ${PATH_PROPOSAL_CRAB}

cp ${WORK_PATH}/${_WASM_CRAB_COMPACT} ${WORK_PATH}/deploy/bin/
cp ${WORK_PATH}/${_WASM_CRAB_COMPRESSED} ${WORK_PATH}/deploy/bin/


# view

ls ${WORK_PATH}/deploy/bin/

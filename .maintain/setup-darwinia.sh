#!/bin/bash
#
#
#

set -xe

BIN_PATH=$(dirname $(readlink -f $0))
WORK_PATH=${BIN_PATH}/../


echo -e '\e[1;32m📥 Installing Cross Compile Toolchain(s)\e[0m'
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
  sh -s -- -y --profile minimal --default-toolchain ${RUST_TOOLCHAIN}
source ~/.cargo/env


echo -e '\e[1;32m🔧 Building Docker Image(s)\e[0m'
docker build -f .maintain/docker/Dockerfile.x86_64-linux-gnu -t x86_64-linux-gnu .
#docker build -f .maintain/docker/Dockerfile.aarch64-linux-gnu -t aarch64-linux-gnu .


cargo install cross \
  --git https://github.com/AurevoirXavier/cross \
  --branch support-multi-sub-targets

rustup target add \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu \
  wasm32-unknown-unknown


if [ -n "${OVERALL_TEST}" ]; then
  cross test --target x86_64-unknown-linux-gnu
  exit 0
fi



# deploy folder is required,
# the x86_64 file will move to this folder,
# the build-image step will use this file to build a docker image
mkdir -p ${WORK_PATH}/deploy/bin


cross build \
  --release \
  --target x86_64-unknown-linux-gnu \
  --sub-targets wasm32-unknown-unknown


## not support now, have build questions.
## https://github.com/fewensa/darwinia/runs/2294261173?check_suite_focus=true#step:4:3092

#RUSTFLAGS='-C link-args=-latomic' SKIP_WASM_BUILD=1 cross build \
#  --no-default-features \
#  --locked \
#  --release \
#  --target aarch64-unknown-linux-gnu

cd ${WORK_PATH}/deploy/bin/

cp ${WORK_PATH}/target/x86_64-unknown-linux-gnu/release/darwinia ${WORK_PATH}/deploy/bin/
chmod +x darwinia
tar cjSf darwinia-x86_64-linux-gnu.tar.bz2 darwinia
mv ${WORK_PATH}/deploy/bin/darwinia ${WORK_PATH}/deploy/

#cp ${WORK_PATH}/target/aarch64-unknown-linux-gnu/release/darwinia ${WORK_PATH}/deploy/bin/
#chmod +x darwinia
#tar cjSf darwinia-aarch64-linux-gnu.tar.bz2 darwinia
#rm -rf ${WORK_PATH}/deploy/bin/darwinia



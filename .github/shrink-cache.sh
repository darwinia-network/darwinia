#!/bin/sh

for p in darwinia
# for p in darwinia polkadot-cli librocksdb-sys wasm-opt-sys
do
  cargo clean -p ${p} --profile ci-dev 2> /dev/null || true
done

# ---

for r in darwinia crab pangoro pangolin
do
  rm -rf target/ci-dev/wbuild/${r}-runtime 2> /dev/null || true
done

for r in polkadot kusama westend rococo
do
  rm -rf target/ci-dev/wbuild/${r}-runtime/target/release 2> /dev/null || true
done

#!/bin/sh

for p in darwinia polkadot-cli librocksdb-sys wasm-opt-sys
do
  cargo clean -p ${p} 2> /dev/null || true
  cargo clean -p ${p} --release 2> /dev/null || true
done
for r in darwinia crab pangoro pangolin
do
  cargo clean -p ${r}-runtime 2> /dev/null || true
  cargo clean -p ${r}-runtime --release 2> /dev/null || true
done

# ---

for r in darwinia crab pangoro pangolin
do
  rm -rf target/debug/wbuild/${r}-runtime 2> /dev/null || true
  rm -rf target/release/wbuild/${r}-runtime 2> /dev/null || true
done

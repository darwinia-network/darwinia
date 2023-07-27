#!/bin/sh

for p in darwinia
do
  cargo clean -p ${p} --profile ci-dev 2> /dev/null || true
done

# ---

for r in darwinia crab pangoro pangolin
do
  rm -rf target/ci-dev/wbuild/${r}-runtime 2> /dev/null || true
done

# for r in polkadot kusama westend rococo
# do
#   rm -rf target/ci-dev/wbuild/${r}-runtime/target/release 2> /dev/null || true
# done

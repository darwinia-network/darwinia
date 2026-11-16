# KTON snapshot freeze

Runtime 7070 freezes Darwinia KTON (asset ID 1026) before the Ethereum migration
snapshot. The upgrade changes the existing asset status from `Live` to `Frozen`.
It preserves account balances, total issuance, asset permissions, approvals,
metadata, retained AccountMigration storage, and staking contract state.

`KtonCallFilter` rejects all native `Assets` calls for asset 1026, including
transfers, approved transfers, minting, burning, refunds, destruction, team
changes, and thawing. It leaves other assets and RING transfers available.
The existing ERC-20 precompile dispatches through this filter; no separate
precompile restriction is introduced. Balance, allowance, supply, and metadata
queries remain available.

The constant-size upgrade migration writes only the existing asset-details
record. An already frozen asset requires no write. The completed collator-count
migration is removed from the upgrade tuple rather than being replayed; the
current collator count is preserved. The local asset-details storage alias mirrors
the private SCALE layout of the locked `pallet-assets` 40.0.0 dependency. Its codec
compatibility and preservation checks must be revisited when upgrading the SDK.

## Execution prerequisites

Before applying this runtime, the migration operator must:

1. Stop new Darwinia-to-Ethereum KTON bridge requests at the contract level and
   confirm that the reverse direction remains closed.
2. Reconcile outstanding requests across both chains. Requests already debited on
   Darwinia but not delivered on Ethereum must be settled or accounted for in an
   explicit migration exception list.
3. Retire the Darwinia staking application's transaction entrypoints and explain
   the migration to users. Retain read access to account balances and staking
   principal; hiding the application does not itself disable its contracts.
4. Run the candidate runtime's Try Runtime checks against the intended chain
   state. The checks reject a missing or destroying KTON asset, require the
   resulting status to be `Frozen`, and verify preservation of the asset record,
   holdings, approvals, metadata, and collator count.

After activation, verify the runtime version, code hash, and frozen KTON status at
a finalized block. Use one finalized block for every native balance and contract
principal read. A published artifact, successful authorization, or a changed
runtime version alone does not establish that the freeze hook has executed.

## Staking and privileged calls

Existing staking principal stays in its contract. Migration accounting assigns
the outstanding principal to its beneficiaries and avoids counting the contract's
custody balance again. Normal stake and withdrawal calls that require a KTON
transfer fail; the EVM transaction reverts its bookkeeping as well. RING rewards
and any separate contract or LP beneficiary accounting require their own policy.

`BaseCallFilter` is not an irreversible restriction on governance. Root and
explicit bypass-filter dispatch paths remain trusted, including paths that can
thaw an asset or write storage. Review scheduled and pending governance actions
before the snapshot and preserve the freeze throughout distribution. Retained
Darwinia KTON must remain unavailable for circulation after its Ethereum allocation
is created. The upgrade does not pause the bridge, retire an application, create
the snapshot, burn tokens, mint Ethereum tokens, or perform governance submission.

## Focused validation

```sh
SKIP_WASM_BUILD=1 cargo test --locked -p darwinia-runtime --features try-runtime --profile ci-dev
SKIP_WASM_BUILD=1 cargo test --locked -p darwinia-runtime --features dev,try-runtime --profile ci-dev
RUSTFLAGS='-C panic=abort' SKIP_WASM_BUILD=1 cargo check --locked -p darwinia-runtime --no-default-features --target wasm32-unknown-unknown --profile ci-dev
cargo +nightly fmt --all -- --check
```

Runtime tests cover the upgrade's idempotency and storage preservation, all
current asset call variants, approved ERC-20 transfers, reads after freezing,
nested utility and proxy dispatch, unrelated transfers, and rollback of
staking-like contract bookkeeping on a rejected KTON transfer.

Local validation on 2026-11-16, using the pinned Rust 1.82.0 toolchain:

- Production runtime with `try-runtime`: 36 tests passed.
- Dev runtime with `try-runtime`: 36 tests passed.
- `wasm32-unknown-unknown` without default features: check passed.
- Nightly formatting and `git diff --check`: passed.
- All 33 locked SDK asset call variants are represented in the filter and dispatch
  tests; external dependency sources and checksums are unchanged.

The `try-runtime` results above are local tests of the migration checks. A Try
Runtime simulation against finalized mainnet state and production Wasm artifact
validation remain prerequisites before an actual upgrade.

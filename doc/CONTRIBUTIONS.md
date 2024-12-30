# Contribution

> [!IMPORTANT]
> This guide assumes that you have a basic understanding of the Rust programming language and the Unix command line.

## Code Structure

| Directory  | Purpose                                                                                                   |
| ---------- | --------------------------------------------------------------------------------------------------------- |
| core       | Core type that is shareable across any Rust project; it operates independently of the runtime.            |
| node       | Client-side code used to execute and build the blockchain network.                                        |
| pallet     | Polkadot-SDK-like pallet code.                                                                            |
| precompile | EVM precompile code.                                                                                      |
| runtime    | Runtime code.                                                                                             |
| tests      | External tests that interact with the node, as opposed to the internal tests conducted with `cargo test`. |
| tool       | Some handy standalone tools, primarily for developers.                                                    |

## Code Style

- Use [EditorConfig](https://editorconfig.org) to ensure consistent coding styles (excluding Rust) across various editors and IDEs.
- Use [rustfmt](https://rust-lang.github.io/rustfmt) to format Rust code.
  - The project is using the stable Rust toolchain by default, which means that some options configured in [.rustfmt.toml](../.rustfmt.toml) are not available. To format the code, use `cargo +nightly fmt`.

## Development Environment

### Setup Toolchain

The default toolchain is specified in the [rust-toolchain.toml](../rust-toolchain.toml). To activate it, ensure that you have [rustup](https://rustup.rs) installed and execute the following command in the project root directory:

```sh
rustup show
```

### Setup Dependencies

This isn't a complete list of dependencies, you should have the basic development environment set up.

Here are the specific dependencies that are required:

```sh
# macOS.
brew install protobuf openssl

# Ubuntu.
apt install protobuf-compiler libssl-dev
```

Feel free to submit a PR to add additional dependencies or OS if you come across any issues.

## Build

### Setup [cargo-make](https://github.com/sagiegurari/cargo-make) (OPTIONAL)

```sh
cargo install cargo-make
```

#### Rule

```sh
# Long command.
cargo make <ACTION>-<MODE>-<OBJECT>-<CHAIN>

# Short command.
cargo make <A><M><O><C>
```

See the [task](#task) section for more details.

### Task

> [!IMPORTANT]
> All commands below should be executed in the project root directory.

```sh
# Format Rust code.
## With cargo-make.
cargo make fmt
## Without cargo-make.
cargo +nightly fmt

# Clippy check. (skip WASM build could speed up the process)
## With cargo-make.
### Long version.
cargo make clippy
### Short version.
cargo make c
## Without cargo-make.
SKIP_WASM_BUILD=1 cargo clippy

# Build the node with all runtime.
## With cargo-make.
### Long version.
cargo make build-node
### Short version.
cargo make bn
## Without cargo-make.
cargo build --locked -p darwinia --features all-runtime

# Build the debug node with Koi runtime. (for Darwinia and Crab, replace the `koi` with the lowercase chain name)
## With cargo-make.
### Long version.
cargo make build-koi
### Short version.
cargo make bk
## Without cargo-make.
cargo build --locked -p darwinia --features koi-runtime

# Build the release node with all runtime.
## With cargo-make.
### Long version.
cargo make build-release-node
### Short version.
cargo make brn
## Without cargo-make.
cargo build --locked -p darwinia --features all-runtime -r

# Build the release node with Koi runtime. (for Darwinia and Crab, replace the `koi` with the lowercase chain name)
## With cargo-make.
### Long version.
cargo make build-release-koi
### Short version.
cargo make brk
## Without cargo-make.
cargo build --locked -p darwinia --features koi-runtime -r

# Build the benchmark node with all runtime.
## With cargo-make.
### Long version.
cargo make build-benchmark
### Short version.
cargo make bb
## Without cargo-make.
cargo build --locked -p darwinia --features all-runtime --features runtime-benchmarks -r

# Build the benchmark node with Koi runtime. (for Darwinia and Crab, replace the `koi` with the lowercase chain name)
## With cargo-make.
### Long version.
cargo make build-benchmark-koi
### Short version.
cargo make bbk
## Without cargo-make.
cargo build --locked -p darwinia --features koi-runtime --features runtime-benchmarks -r

# Run dev Koi node. (for Darwinia and Crab, replace the `koi` with the lowercase chain name)
## With cargo-make.
### Long version.
cargo make run-dev-koi
### Short version.
cargo make rdk
## Without cargo-make.
cargo run --locked -p darwinia --features koi-runtime -- --unsafe-rpc-external --tmp --rpc-cors all --rpc-methods unsafe --alice --unsafe-force-node-key-generation --chain koi-dev

# Run release dev Koi node. (for Darwinia and Crab, replace the `koi` with the lowercase chain name)
## With cargo-make.
### Long version.
cargo make run-release-dev-koi
### Short version.
cargo make rrdk
## Without cargo-make.
cargo run --locked -p darwinia --features koi-runtime -r -- --unsafe-rpc-external --tmp --rpc-cors all --rpc-methods unsafe --alice --unsafe-force-node-key-generation --chain koi-dev

# Run benchmark Koi node. (for Darwinia and Crab, replace the `koi` with the lowercase chain name)
## With cargo-make.
### Long version.
cargo make run-benchmark-koi
### Short version.
cargo make rbk
## Without cargo-make.
cargo run --locked -p darwinia --features koi-runtime --features runtime-benchmarks -r -- benchmark pallet --header .maintain/license-header --heap-pages 4096 --chain koi-dev --output runtime/koi/src/weights --pallets \* --extrinsic \* --steps 50 --repeat 20
```

## Public RPC Endpoint

### Darwinia

- `://rpc.darwinia.network` (`https` or `wss`)
- `://darwinia-rpc.dwellir.com` (`https` or `wss`)

### Crab

- `://crab-rpc.darwinia.network` (`https` or `wss`)
- `://darwiniacrab-rpc.dwellir.com` (`https` or `wss`)

### Koi

- `://koi-rpc.darwinia.network` (`https` or `wss`)

### Pangoro

- https://fraa-flashbox-2871-rpc.a.stagenet.tanssi.network

## EVM Specification

### Chain ID

- [Koi - `701`](https://chainlist.org/chain/701)
- [Crab - `44`](https://chainlist.org/chain/44)
- [Darwinia - `46`](https://chainlist.org/chain/46)

### Tracing Node Endpoint

- Darwinia: `ws://c1.darwinia2.darwinia.network:9944`
- Crab: `ws://c1.crab2.darwinia.network:9944`
- Koi: `ws://g3.testnets.darwinia.network:9942`

### Devnet Built-in Accounts

```json
[
	{
		"name": "Alith",
		"p": "0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f",
		"s": "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133"
	},
	{
		"name": "Baltathar",
		"p": "0x033bc19e36ff1673910575b6727a974a9abd80c9a875d41ab3e2648dbfb9e4b518",
		"s": "0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b"
	},
	{
		"name": "Charleth",
		"p": "0x0234637bdc0e89b5d46543bcbf8edff329d2702bc995e27e9af4b1ba009a3c2a5e",
		"s": "0x0b6e18cafb6ed99687ec547bd28139cafdd2bffe70e6b688025de6b445aa5c5b"
	},
	{
		"name": "Dorothy",
		"p": "0x02a00d60b2b408c2a14c5d70cdd2c205db8985ef737a7e55ad20ea32cc9e7c417c",
		"s": "0x39539ab1876910bbf3a223d84a29e28f1cb4e2e456503e7e91ed39b2e7223d68"
	},
	{
		"name": "Ethan",
		"p": "0x025cdc005b752651cd3f728fb9192182acb3a9c89e19072cbd5b03f3ee1f1b3ffa",
		"s": "0x7dce9bc8babb68fec1409be38c8e1a52650206a7ed90ff956ae8a6d15eeaaef4"
	},
	{
		"name": "Faith",
		"p": "0x037964b6c9d546da4646ada28a99e34acaa1d14e7aba861a9055f9bd200c8abf74",
		"s": "0xb9d2ea9a615f3165812e8d44de0d24da9bbd164b65c4f0573e1ce2c8dbd9c8df"
	}
]
```

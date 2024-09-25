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
cargo make build-node-koi
### Short version.
cargo make bnk
## Without cargo-make.
cargo build --locked -p darwinia --features koi-runtime

# Build the release node with all runtime.
## With cargo-make.
### Long version.
cargo make build-release-node
### Short version.
cargo make brn
## Without cargo-make.
cargo build -r --locked -p darwinia --features all-runtime

# Build the release node with Koi runtime. (for Darwinia and Crab, replace the `koi` with the lowercase chain name)
## With cargo-make.
### Long version.
cargo make build-release-node-koi
### Short version.
cargo make brnk
## Without cargo-make.
cargo build -r --locked -p darwinia --features koi-runtime

# Run dev Koi node. (for Darwinia and Crab, replace the `koi` with the lowercase chain name)
## With cargo-make.
### Long version.
cargo make run-dev-koi

```

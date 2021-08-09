[![CI](https://travis-ci.org/darwinia-network/darwinia.svg)](https://travis-ci.org/darwinia-network/darwinia])
[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)


# ![Logo](https://github.com/darwinia-network/rfcs/raw/master/logo/darwinia.png)

**As an open cross-chain bridge protocol based on Substrate, Darwinia aims to build the Internet of Tokens, including decentralized tokens swap, exchange, and market.**

👉 *The official Documents for [The Darwinia Network](https://darwinia.network/)*<br>
👉 *The official Documents for [The Crab Network](https://docs.crab.network/)*<br>
👉 *The official communication zoom, if you have any question? [Darwinia Room](https://matrix.to/#/#darwinia:matrix.org)*<br>

## Building from Source

> NOTE: Substrate development for windows is not properly supported! It is highly recommend to use Windows Subsystem Linux (WSL) and follow the instructions for Ubuntu/Debian. Check out [Getting Started on Windows](https://substrate.dev/docs/en/knowledgebase/getting-started/windows-users) for more detail.

For Unix-based user, just grab the source code and build it. Make sure you have Rust and the support software installed.

Install Rust first:

```sh
curl https://sh.rustup.rs -sSf | sh
```

```sh
rustup default nightly
rustup target add wasm32-unknown-unknown
```

Additional packages may also be required:

- Linux:

	```sh
	sudo apt install cmake pkg-config libssl-dev git clang libclang-dev
	```

- Mac:

	```sh
	brew install cmake pkg-config openssl git llvm
	```

Then, clone the Darwinia source code and build the target:

```sh
# Fetch the code
git clone https://github.com/darwinia-network/darwinia.git
cd darwinia

# Build the node (The first build will be long (~30min))
cargo build --release
```

Run a development chain with:

```sh
cargo run --release -- --dev
```

## Joining Network

- [Learn how to join the Crab Network](https://docs.crab.network/crab-tut-node)
- [Learn how to join the Darwinia Network](https://docs.darwinia.network/docs/en/wiki-tut-node)

## Contributing Guidelines

[CONTRIBUTION](CONTRIBUTING.adoc)

## License

[LICENSE](https://github.com/darwinia-network/darwinia/blob/master/LICENSE)

# Evolution Land Chain
Land based on Parity Substrate.

## Fresh start
If your device is clean (such as a fresh cloud VM) you can use this script, otherwise, proceed with the Initial Setup.

```bash
./setup.sh
```
Then proceed to the Running instructions or follow the instructions below for the manual setup.


## Initial Setup
```bash
curl https://sh.rustup.rs -sSf | sh
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update stable
cargo install --git https://github.com/alexcrichton/wasm-gc
```
You will also need to install the following packages:

Linux:
```bash
sudo apt install cmake pkg-config libssl-dev git clang libclang-dev
```

Mac:
```bash
brew install cmake pkg-config openssl git llvm
```

Building
```bash
./build.sh
cargo build --release
```

## Running
```bash
./target/release/evolutionland --dev
```

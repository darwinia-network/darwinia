# Darwinia AppChain
Application chain in Darwinia network based on substrate framework. For detail introduction, view [RFC for Darwinia Appchain](https://github.com/evolutionlandorg/ELIPs/blob/master/rfcs/zh_CN/0006-dawinia-appchain.md)

The next coming land will use the Darwinia AppChain to build and develop.


## Start

proceed to the Running instructions or follow the instructions below for the manual setup.

### Initial Setup
```bash
./init.sh
```
Or, you can run scripts step by step, like the following:
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


### Building
```bash
./build.sh
cargo build --release
```

Running
```bash
./target/release/land-chain --dev
```

Play with gui, open

[https://polkadot.js.org/apps/#/settings](https://polkadot.js.org/apps/#/settings)

And select the local node (127.0.0.1), please note that for the current GUI version, custom struct&tpyes must be configured before viewing.

Go to [https://polkadot.js.org/apps/#/settings/developer](https://polkadot.js.org/apps/#/settings/developer)

And copy the content in
```
./types.json
```

to the type definitions text area.


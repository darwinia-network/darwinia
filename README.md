# Darwinia AppChain
Application chain in Darwinia network based on substrate framework. For detail introduction, view [RFC for Darwinia Appchain](https://github.com/evolutionlandorg/ELIPs/blob/master/rfcs/zh_CN/0006-dawinia-appchain.md)

The next coming land will use the Darwinia AppChain to build and develop.

![Darwinia AppChain Logo](https://raw.githubusercontent.com/evolutionlandorg/ELIPs/master/logo/darwinia_appchain.png)

# Road Map
[Road Map](https://hackmd.io/iofzom6eRe-a7fOQeoZSQQ)



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
./target/release/darwinia-appchain --dev
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

### Running Local Testnet (default: Alice and Bob)
first build:
```bash
./build.sh
cargo build --release
```

#### Alice Starts first
Alice should run t his command from ${PATH_TO_DARWINIA_APPCHAIN_ROOT}:
```bash
cd {path_to_darwinia_appchain_root}
./target/release/darwinia-appchain \
--base-path /tmp/alice \
--chain=local \
--key //Alice \
--port 30333 \
--validator \
--name AliceDarwiniaNode \
--telemetry-url ws://telemetry.polkadot.io:1024
```

#### Bob Joins In
Now that Alice's node is up and running, Bob can join the network by bootstrapping from her node. His command will look very similar.
```bash
./target/release/darwinia-appchain \
--base-path /tmp/bob \
--chain=local \
--key //Bob \
--port 30334 \
--validator \
--name BobDarwiniaNode
-- botenodes /ip4/<Alices IP Address>/tcp/<Alices Port>/p2p/<Alices Node ID> \
--telemetry-url ws://telemetry.polkadot.io:1024
```

- If these two nodes are running on the same physical machine, Bob MUST specify a different `--base-path` and `--port`.
- Bob has added the `--bootnodes` flag and specified a single boot node, namely Alice's. He must correctly specify these three pieces of information which Alice can supply for him.
  - Alice's IP Address in the form `192.168.1.1`
  - Alice's Port, probably 30333
  - Alice's node ID, copied from her log output. (proberbly`Qmc1RbjHGWGY4E4gkEGbSX3RMcfqbmZwZumga1uNaYQvU5` in the output above.)
  
- How to figure out Alice's Node ID
you can find Alice's node id in terminal outputs when AliceNode starts:
```bash
2019-05-24 16:45:33 Darwinia AppChain
2019-05-24 16:45:33   version 1.0.0-fdf0687-x86_64-macos
2019-05-24 16:45:33   by Evolution Land <hello@evolution.land>, 2017, 2018
2019-05-24 16:45:33 Chain specification: Local Testnet
2019-05-24 16:45:33 Node name: AliceNode
2019-05-24 16:45:33 Roles: AUTHORITY
2019-05-24 16:45:33 Best block: #19
2019-05-24 16:45:33 Using default protocol ID "sup" because none is configured in the chain specs
2019-05-24 16:45:33 Local node identity is: Qmc1RbjHGWGY4E4gkEGbSX3RMcfqbmZwZumga1uNaYQvU5
```  

If all is going well, after a few seconds, the nodes should peer together and start producing blocks. You should see some lines like:
for Alice:
```bash
2019-05-24 16:59:54 Idle (0 peers), best: #45 (0xf5d0…5549), finalized #0 (0xe6fe…6664), ⬇ 0 ⬆ 0
2019-05-24 16:59:54 Discovered external node address: /ip4/192.168.2.185/tcp/30333/p2p/Qmc1RbjHGWGY4E4gkEGbSX3RMcfqbmZwZumga1uNaYQvU5
2019-05-24 16:59:59 Idle (1 peers), best: #45 (0xf5d0…5549), finalized #0 (0xe6fe…6664), ⬇ 0.5kiB/s ⬆ 0.5kiB/s
2019-05-24 17:00:00 Starting consensus session on top of parent 0xf5d07ea0778109602f93c40bc9586e331355e68cfcf3deb1721a65258c545549
2019-05-24 17:00:00 Prepared block for proposing at 46 [hash: 0xc394f4f58614b4bf4ce45ef593b04cce5953eead47e72286d5d288f5f97f9c7d; parent_hash: 0xf5d0…5549; extrinsics: [0xfc6d…480f]]
2019-05-24 17:00:00 Pre-sealed block for proposal at 46. Hash now 0x2b21cf6d11e6978e548abcdc80da20bb7cf6eca8ed352b06464d7e3b43153c40, previously 0xc394f4f58614b4bf4ce45ef593b04cce5953eead47e72286d5d288f5f97f9c7d.
2019-05-24 17:00:00 Imported #46 (0x2b21…3c40)
2019-05-24 17:00:04 Idle (1 peers), best: #46 (0x2b21…3c40), finalized #0 (0xe6fe…6664), ⬇ 15 B/s ⬆ 0.1kiB/s
```

for Bob:
```bash
2019-05-24 16:59:53 Idle (0 peers), best: #45 (0xf5d0…5549), finalized #0 (0xe6fe…6664), ⬇ 0 ⬆ 0
2019-05-24 16:59:54 Discovered external node address: /ip4/192.168.2.185/tcp/30334/p2p/QmPQdzXx95sex3wtrUFPP1oS4AWRHBsbhMnAJmfKJqX5Ly
2019-05-24 16:59:58 Libp2p => Random Kademlia query has yielded empty results
2019-05-24 16:59:58 Idle (1 peers), best: #45 (0xf5d0…5549), finalized #0 (0xe6fe…6664), ⬇ 0.5kiB/s ⬆ 0.5kiB/s
2019-05-24 17:00:00 Imported #46 (0x2b21…3c40)
2019-05-24 17:00:02 Libp2p => Random Kademlia query has yielded empty results
2019-05-24 17:00:03 Idle (1 peers), best: #46 (0x2b21…3c40), finalized #0 (0xe6fe…6664), ⬇ 0.1kiB/s ⬆ 15 B/s
2019-05-24 17:00:08 Idle (1 peers), best: #46 (0x2b21…3c40), finalized #0 (0xe6fe…6664), ⬇ 63 B/s ⬆ 73 B/s
2019-05-24 17:00:10 Starting consensus session on top of parent 0x2b21cf6d11e6978e548abcdc80da20bb7cf6eca8ed352b06464d7e3b43153c40
2019-05-24 17:00:10 Prepared block for proposing at 47 [hash: 0xeba1f1859e37f0761b096e4a33026067a2f36a31ab8a01a503536c8603cacc27; parent_hash: 0x2b21…3c40; extrinsics: [0xc66c…35b9]]
2019-05-24 17:00:10 Pre-sealed block for proposal at 47. Hash now 0xe10a87970b7e4d3014d8faaa995e034cb4427d6fe0958c917f2ab572dc721969, previously 0xeba1f1859e37f0761b096e4a33026067a2f36a31ab8a01a503536c8603cacc27.
```
The first line shows that Bob has discovered Alice on the network. The second shows that he has peered with her (1 peers), they have produced a block (best: #1 (0xf5d0…5549)), and the block is not finalized (finalized #0 (0xe6fe…6664)).


#### View On Telemetry
then you can find your Node displayed on [Telemetry](https://telemetry.polkadot.io/#/Local%20Testnet)
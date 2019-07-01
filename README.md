
# Darwinia Relay Chain

![Darwinia Logo](https://github.com/darwinia-network/rfcs/raw/master/logo/darwinia.png)


Darwinia Relay Chain is the hub relay chain connecting different Darwinia AppChains and can be connected to Polkadot as a Polkadot Parachain.

It could have two models, the Solo model and the Polkadot model. For more details, go to [RFC-0007](  https://github.com/darwinia-network/rfcs/blob/master/zh_CN/0007-dawinia-token-staking-model.md#solo%E6%A8%A1%E5%BC%8F 
)

# Architecture

![Darwinia Architecture](https://github.com/darwinia-network/rfcs/raw/master/zh_CN/images/0007-darwinia-architecture.jpeg)


# Road Map
[Road Map](ROADMAP.md)

# Community
Join the community if you have any other questions:

[+darwinia:matrix.org](https://matrix.to/#/+darwinia:matrix.org)

Or

[Riot.im](https://riot.im/app/#/group/+darwinia:matrix.org)

# Applications and Examples

- [Evolution Land](https://www.evolution.land/) and Project [Github](https://github.com/evolutionlandorg), a virtual management game based on blockchain and autonomy.
- [Darwinia AppChain](https://github.com/darwinia-network/darwinia-appchain) Application Chain SDK Suite
- [Darwinia Bridge](https://github.com/darwinia-network/darwinia-bridge) Darwinia Bridge Parachain and Tools to connect to other chains such as Ethereum, TRON and EOS etc.
- More are coming...

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
./target/release/darwinia --dev
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
./target/release/darwinia \
--base-path /tmp/alice \
--key //Alice \
--port 30333 \
--validator \
--name AliceDarwiniaNode \
--rpc-external \
--ws-external
```
`rpc-external` and `ws-external` flags are optional.

#### Bob Joins In
Now that Alice's node is up and running, Bob can join the network by bootstrapping from her node. His command will look very similar.
```bash
./target/release/darwinia \
--base-path /tmp/bob \
--key //Bob \
--port 30334 \
--validator \
--name BobDarwiniaNode \
--bootnodes /ip4/<Alices IP Address>/tcp/<Alices Port>/p2p/<Alices Node ID> \
--telemetry-url ws://telemetry.polkadot.io:1024 \
--rpc-external \
--ws-external
```

- If these two nodes are running on the same physical machine, Bob MUST specify a different `--base-path` and `--port`.
- Bob has added the `--bootnodes` flag and specified a single boot node, namely Alice's. He must correctly specify these three pieces of information which Alice can supply for him.
  - Alice's IP Address in the form `192.168.1.1`
  - Alice's Port, probably 30333
  - Alice's node ID, copied from her log output. (proberbly`Qmc1RbjHGWGY4E4gkEGbSX3RMcfqbmZwZumga1uNaYQvU5` in the output above.)
  
- How to figure out Alice's Node ID
you can find Alice's node id in terminal outputs when AliceNode starts:
```bash
2019-06-29 18:22:56 Darwinia POC-1 Node
2019-06-29 18:22:56   version 0.1.0-9d2ab05-x86_64-macos
2019-06-29 18:22:56   by Darwinia Network, 2017-2019
2019-06-29 18:22:56 Chain specification: Darwinia POC-1 Testnet
2019-06-29 18:22:56 Node name: alexxxxx
2019-06-29 18:22:56 Roles: AUTHORITY
2019-06-29 18:22:56 Highest known block at #20
2019-06-29 18:22:56 Using default protocol ID "sup" because none is configured in the chain specs
2019-06-29 18:22:56 Local node identity is: QmdSidct8sAvQbMZE7YoU5hk3bPnpmQUN6BCo3Vgd8BZAG
2019-06-29 18:22:56 Libp2p => Random Kademlia query has yielded empty results
2019-06-29 18:22:56 Using authority key 5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu
2019-06-29 18:22:56 Running Grandpa session as Authority 5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu
2019-06-29 18:22:57 Libp2p => Random Kademlia query has yielded empty results
2019-06-29 18:22:59 Libp2p => Random Kademlia query has yielded empty results
2019-06-29 18:23:01 Idle (0 peers), best: #20 (0xd7ab…9164), finalized #20 (0xd7ab…9164), ⬇ 0 ⬆ 0
^C2019-06-29 18:23:03 Libp2p => Random Kademlia query has yielded empty results
```  

If all is going well, after a few seconds, the nodes should peer together and start producing blocks. You should see some lines like:
for Alice:
```bash
2019-06-29 18:25:06 Libp2p => Random Kademlia query has yielded empty results
2019-06-29 18:25:08 Idle (0 peers), best: #20 (0xd7ab…9164), finalized #20 (0xd7ab…9164), ⬇ 0 ⬆ 0
2019-06-29 18:25:10 Libp2p => Random Kademlia query has yielded empty results
2019-06-29 18:25:13 Idle (1 peers), best: #20 (0xd7ab…9164), finalized #20 (0xd7ab…9164), ⬇ 0.4kiB/s ⬆ 0.4kiB/s
2019-06-29 18:25:13 Discovered new external address for our node: /ip4/192.168.110.246/tcp/20222/p2p/QmdSidct8sAvQbMZE7YoU5hk3bPnpmQUN6BCo3Vgd8BZAG
2019-06-29 18:25:16 Imported #21 (0x737a…ea93)
2019-06-29 18:25:18 Idle (1 peers), best: #21 (0x737a…ea93), finalized #20 (0xd7ab…9164), ⬇ 1.2kiB/s ⬆ 0.9kiB/s
2019-06-29 18:25:20 Starting consensus session on top of parent 0x737a8e622371b9c33c7ed284ce5ed422b81e7fb02d1397dc1c1676dae5efea93
2019-06-29 18:25:20 Prepared block for proposing at 22 [hash: 0x3d0c8c24c5208432cfe87658b0cfe04db0b9ec0a16d4cd8b2e2cc40e8f03ba6a; parent_hash: 0x737a…ea93; extrinsics: [0x842d…8482, 0xfc43…52da]]
2019-06-29 18:25:20 Pre-sealed block for proposal at 22. Hash now 0x024b81b36f673b2c3338cc4665cb42190b82fc0342dba30b6bc2570080b80d78, previously 0x3d0c8c24c5208432cfe87658b0cfe04db0b9ec0a16d4cd8b2e2cc40e8f03ba6a.
2019-06-29 18:25:20 Imported #22 (0x024b…0d78)
2019-06-29 18:25:23 Idle (1 peers), best: #22 (0x024b…0d78), finalized #22 (0x024b…0d78), ⬇ 1.2kiB/s ⬆ 1.1kiB/s
```

for Bob:
```bash
2019-06-29 18:25:12 Highest known block at #20
2019-06-29 18:25:12 Using default protocol ID "sup" because none is configured in the chain specs
2019-06-29 18:25:12 Local node identity is: QmRJGKPvX76KZfGGhzUo6xTcb9UM9RwEM19bKqwxBDS5Vp
2019-06-29 18:25:12 Libp2p => Random Kademlia query has yielded empty results
2019-06-29 18:25:12 Unable to bind server to 127.0.0.1:9944. Trying random port.
2019-06-29 18:25:12 Using authority key 5GoNkf6WdbxCFnPdAnYYQyCjAKPJgLNxXwPjwTh6DGg6gN3E
2019-06-29 18:25:12 Running Grandpa session as Authority 5GoNkf6WdbxCFnPdAnYYQyCjAKPJgLNxXwPjwTh6DGg6gN3E
2019-06-29 18:25:13 Discovered new external address for our node: /ip4/192.168.110.246/tcp/20223/p2p/QmRJGKPvX76KZfGGhzUo6xTcb9UM9RwEM19bKqwxBDS5Vp
2019-06-29 18:25:16 Starting consensus session on top of parent 0xd7ab506dd6e388e8ccfcb62c6e64689be0a26fa1a0cfc4027d05c32d55a79164
2019-06-29 18:25:16 Prepared block for proposing at 21 [hash: 0x5e2f5d1ea92bd60087fd30f20a5917d1ca60ec5d5d32190f752bcf2f826c3a5c; parent_hash: 0xd7ab…9164; extrinsics: [0x977d…1cc9, 0x73e9…bfed]]
2019-06-29 18:25:16 Pre-sealed block for proposal at 21. Hash now 0x737a8e622371b9c33c7ed284ce5ed422b81e7fb02d1397dc1c1676dae5efea93, previously 0x5e2f5d1ea92bd60087fd30f20a5917d1ca60ec5d5d32190f752bcf2f826c3a5c.
2019-06-29 18:25:16 Imported #21 (0x737a…ea93)
2019-06-29 18:25:17 Idle (1 peers), best: #21 (0x737a…ea93), finalized #20 (0xd7ab…9164), ⬇ 1.4kiB/s ⬆ 1.7kiB/s
2019-06-29 18:25:20 Imported #22 (0x024b…0d78)
2019-06-29 18:25:22 Idle (1 peers), best: #22 (0x024b…0d78), finalized #22 (0x024b…0d78), ⬇ 1.0kiB/s ⬆ 1.1kiB/s
2019-06-29 18:25:24 Starting consensus session on top of parent 0x024b81b36f673b2c3338cc4665cb42190b82fc0342dba30b6bc2570080b80d78
2019-06-29 18:25:24 Prepared block for proposing at 23 [hash: 0x8947ad3cebd65d9de60166743eae1dd198cb10751764faefba6c71be8d5ccf5f; parent_hash: 0x024b…0d78; extrinsics: [0x5b14…3439, 0xec49…f2f5]]
2019-06-29 18:25:24 Pre-sealed block for proposal at 23. Hash now 0xcd96f0558e8c2f2a38e680f2eb280ccbb1422f6df3bc3dfa306ef89c943f8a05, previously 0x8947ad3cebd65d9de60166743eae1dd198cb10751764faefba6c71be8d5ccf5f.
2019-06-29 18:25:24 Imported #23 (0xcd96…8a05)
2019-06-29 18:25:27 Idle (1 peers), best: #23 (0xcd96…8a05), finalized #23 (0xcd96…8a05), ⬇ 1.0kiB/s ⬆ 1.1kiB/s
2019-06-29 18:25:28 Imported #24 (0xe4b1…e584)
```
The first line shows that Bob has discovered Alice on the network. The second shows that he has peered with her (1 peers), they have produced a block (best: #1 (0xf5d0…5549)), and the block is not finalized (finalized #0 (0xe6fe…6664)).


#### View On Telemetry
then you can find your Node displayed on [Telemetry](https://telemetry.polkadot.io/#/Local%20Testnet)


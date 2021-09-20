# Darwinia RoadMap

## Introduction

As an open cross-chain bridge protocol based on Substrate, Darwinia focuses on the construction of future Internet of Tokens, including decentralized tokens swap, exchange, and market.

## Development RoadMap

### The specifications of the project including:

1. Darwinia bridge chain mainet core features
    - Dual token model design and implementation, the native tokens are RING and KTON.
    - RING token for payment and gas fees, can be locked to issue KTON
    - KTON can only be created by locking RING, the amount issued depends on how long the RING will be locked.
2. The staking model.
    - Withdraw income support to KTON Token before transferring
    - Support Treasury runtime within the staking
    - Normal staking using locked KTON as voting tickets, and income are paid in RING
3. Darwinia Relay Design and Implementation.
    - The features will follow the design from Parity Bridge.
    - The NFT cross-chain code protocol will follow the proposal from the Drawinia Network RFC-0005
    - The current RING/KTON tokens on Evolution Land will use this feature to finish the cross-chain transfer.
    - The current game NFTs on Evolution Land will use this feature to transfer between Ethereum/Tron through Darwinia Network.
4. Javascript SDK
    - Api derives based on substrate and polkadot.js
    - Apps entry hosting.
5. Token DEX and NFT Markets


## Current Status
Currently we are developing based on Substrate 2.0 alpha, prepare for the Crab Network.

After Crab Network and Substrate 2.0 stable, we will prepare for the launch of Darwinia mainet.

## Milestones

### Trilobita Testnet (2019-07 Launched)
Include RING/KTON token runtime and Grigott runtime(Issuing KTON buy locking RING).

- Features and functions
    - Research on the tokens economic and staking protocol of Darwinia Network, and finalized the design paper.
    - Develop several SRML modules based on Substrate, and run a public testnet.
    - Develop a web wallet based on polkadot-js.
    - Develop a blockchain browser for the testnet to view blocks and transactions.
- Deliverables
    - Docker container running a Substrate node with a simple native token system runtime, can connect to testnet and syncing blocks.
    - A design paper introducing this project, and the technical structure of it, including token economics and staking.
    - Telemetry demonstrate the nodes status of the testnet.
    - A prototype web wallet for users and tester to play with.

### Crayfish Testnet (2019-09 Launched)
Including staking runtime.

- Features and functions
    - A simple blockchain browser.
    - Staking runtime and functions.
    - Web wallet updates to this new functions
    - A simple blockchain browser.
    - Testnet tokens faucet.
- Deliverables
    - Docker container running a darwinia node with staking runtime included, can connect to testnet and syncing blocks.
    - Running node can get free tokens from faucet, and testing validator functions, running as validators. And normal users can support validators by nominating.
    - Users and view the blockchain data and extrinsics using blockchain browser.

### Icefrog Testnet (2019-11 Launched)
Release candidate for mainnet launch(2019Q4), with runtimes including cross-chain NFT bridge, fungible token bridge between Ethereum and Tron, and experimental contract module.

- Features and functions
    - Cross-chain NFT encoding and bridging Ethereum/TRON testnet and the POC-3 Testnet.
    - Cross-chain fungible token bridging, from Ethereum ERC-20 and TRON TRC-20 to Darwinia’s native tokens, e.g. RING and KTON.
    - Experimental contract runtime support. (Using pDSL for testing and experimental, only support command line)
    - Upgraded web wallet and blockchain browser with better user experience.
- Deliverables
    - Docker container running a darwinia node with NFT/Token swapping runtime included, can connect to testnet and syncing blocks.
    - User can use web wallet to test the NFT and token bridging, e.g. transferring a NFT token from Ethereum testnet to Tron testnet. (Evolution Land’s alpha version, can be used as a scenario)
    - Documents about how to deploy a sample contract on the experimental contract model
    - Blockchain browser can view the NFT token’s encoding ids, and search NFT by id.

### Crab Network (Expected 2020 03)
Canary Network of Darwinia.


### Darwinia Mainnet (Expected 2020 Q2)
There could more testnet for testing and security auditing and preparing for the mainnet launch at Q4 2019.


### Token Dex and Swap Protocol
[TBD]

- Dai Token Bridge
- Uniswap alike swap protocol
- Other DEX protocol design and implementation

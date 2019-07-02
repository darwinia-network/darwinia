# Darwinia Network RoadMap

## Introduction

Darwinia Network provides game developers the scalability, cross-chain interoperability, and NFT identifiability, with seamless integrations to Polkadot, bridges to all major blockchains, and on-chain RNG services.
 
## Development RoadMap

### The specifications of the project including:

1. Dual token model design and implementation, the native tokens are RING and KTON.
    - RING token for payment and gas fees, can be locked to issue KTON
    - KTON can only be created by locking RING, the amount issued depends on how long the RING will be locked.
    - The draft of the specification is now located in RFC-0007 (English version is TBD)
2. The staking allocation in the solo model.
    - Withdraw income support to KTON Token before transferring
    - Support Treasury runtime within the staking
    - Normal staking using locked KTON as voting tickets, and income are paid in RING
3. Migrating current contract runtime, and support paying gas fee using RING token
4. Cross-chain bridges supporting token cross-chain transferring between Ethereum(ERC-20)/Tron(TRC-20) and Darwinia network, fungible token and non-fungible tokens will be supported.
    - The features will follow the design from Parity Bridge.
    - The NFT cross-chain code protocol will follow the proposal from the Drawinia Network RFC-0005
    - The current RING/KTON tokens on Evolution Land will use this feature to finish the cross-chain transfer.
    - The current game NFTs on Evolution Land will use this feature to transfer between Ethereum/Tron through Darwinia Network.
5 Tools and Documents
    - Web wallet providing user interface for users to interact with. M1
    - Blockchain browser: help common user check transaction and data M2
    - Tutorials for users and token holders about staking and nominating.
    - Documents for validators to setup nodes and join the network

Currently we are developing based on Substrate 1.0 rc, to setup the initial POC-1 testnet.  Later we will refactor code to comply with latest Substrate 2.0 design and trait, as Substrate 2.0 has no stable release yet, we'd start from what is there (the POC-1 testnet release at the time of writing).  Changes to Substrate 2.0 might cause refactorings and delay the plan.

### Milestones

- Start: already started, can view the current status of the project and development here:
    - https://github.com/darwinia-network/darwinia
    - https://github.com/darwinia-network/darwinia/milestone/1 (WIP)
    
- M1 (2 weeks): Darwinia PoC1 Testnet.  Include RING/KTON token runtime and Grigott runtime(Issuing KTON buy locking RING). 
    - Features and functions
        - Research on the tokens economic and staking protocol of Darwinia Network, and finalized the design paper
        - Develop several SRML modules based on Substrate, and run a public testnet.
        - Develop a web wallet based on polkadot-js.
        - Develop a blockchain browser for the testnet to view blocks and transactions.
    - Deliverables
        - Docker container running a Substrate node with a simple native token system runtime, can connect to testnet and syncing blocks.
        - A design paper introducing this project, and the technical structure of it, including token economics and staking.
        - Telemetry demonstrate the nodes status of the testnet
        - A prototype web wallet for users and tester to play with.
        
- M2 (4 weeks): Darwinia PoC2 Testnet. Including staking runtime.
    - Features and functions
        - A simple blockchain browser.
        - Staking runtime and functions.
        - Web wallet updates to this new functions
        - A simple blockchain browser.
        - Testnet tokens faucet.
    - Deliverables
        - Docker container running a substrate node with staking runtime included, can connect to testnet and syncing blocks.
        - Running node can get free tokens from faucet, and testing validator functions, running as validators. And normal users can support validators by nominating.
        - Users and view the blockchain data and extrinsics using blockchain browser.
        
- M3 (6 weeks): Darwinia PoC3 Testnet. Release candidate for mainnet launch(2019Q4), with runtimes including cross-chain NFT bridge, fungible token bridge between Ethereum and Tron, and experimental contract module.
    - Features and functions
        - Cross-chain NFT encoding and bridging Ethereum/TRON testnet and the POC-3 Testnet.
        - Cross-chain fungible token bridging, from Ethereum ERC-20 and TRON TRC-20 to Darwinia’s native tokens, e.g. RING and KTON.
        - Experimental contract runtime support. (Using pDSL for testing and experimental, only support command line)
        - Upgraded web wallet and blockchain browser with better user experience.
    - Deliverables
        - Docker container running a substrate node with NFT/Token swapping runtime included, can connect to testnet and syncing blocks.
        - User can use web wallet to test the NFT and token bridging, e.g. transferring a NFT token from Ethereum testnet to Tron testnet. (Evolution Land’s alpha version, can be used as a scenario)
        - Documents about how to deploy a sample contract on the experimental contract model
        - Blockchain browser can view the NFT token’s encoding ids, and search NFT by id.
 
After that, there could more testnet for testing and security auditing and preparing for the mainnet launch at Q4 2019. 


### [Mainnet]
2019 Q4

## Darwinia AppChain Beta Version
2020 Q1
Darwinia AppChain SDK Suite Beta version release.

2020 Q1
Evolution Land's adoption use case (testing version)
    
## Evolution Land's new land launched base on Darwinia AppChain
2020 Q2    

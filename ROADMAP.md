# Darwinia Network RoadMap

## Darwinia Relay Chain RoadMap
> Darwinia Relay Chain is the hub chain of Darwinia Network
> 


### [POC-1 Testnet] Solo Model Testnet
DEADLINE: 2019-06-30

- Basic RING token system [RFC-0007](https://github.com/darwinia-network/rfcs/blob/master/zh_CN/0007-dawinia-token-staking-model.md#solo%E6%A8%A1%E5%BC%8F)
- Gas Model [RFC-0002](https://github.com/darwinia-network/rfcs/blob/master/zh_CN/0002-darwinia-gas-model.md)
- Staking system [RFC-0007](https://github.com/darwinia-network/rfcs/blob/master/zh_CN/0007-dawinia-token-staking-model.md#solo%E6%A8%A1%E5%BC%8F)
- KTON system 
- Devops Tools
- Web Wallet[WIP]
- Blockchain Browser[WIP] 
- Contract Model 
    1. 付费 （开发者付费/从抵押kton的分成优先付费）
    2. built-in account （系统分红）
    3. gas meter（目前有的方案）
    
- Staking Model Research
    1. validator
    2. collactor

功能包括：
 - 用ring抵押获得kton
 - 使用kton解锁抵押的ring
 - 抵押后的kton作为权益，可以(实时？)享受系统分成
 - kton分得的ring可以用来支付手续费(gas)
 - 如果kton分红不足以抵扣gas，则使用ring购买
 - 开发者可以代替用户付费（借鉴波场）


要求：
- 优先复用现有trait
- 优先解耦
- 进度优先，代码可以未来重构

### [POC-2 Testnet] WIP
2019 Q3

### [Mainnet]
2019 Q1

## Darwinia AppChain RoadMap Testnet
2020 Q1
Darwinia AppChain SDK Suite Beta version release.

2020 Q1
Evolution Land's adoption use case (testing version)
    
## Evolution Land's new land launched base on Darwinia AppChain
2020 Q2    

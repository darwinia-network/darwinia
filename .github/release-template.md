Darwinia
===


## [0.11.6] - 2021-11-23
| Network  | Native Runtime | Upgrade Priority |
| :------: | :------------: | :--------------: |
| Darwinia |      1160      |       HIGH       |
|   Crab   |      1160      |       HIGH       |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                      Download                                                       |
| :---: | :----: | :---: | :---: | :-----------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.zst](https://github.com/darwinia-network/darwinia/releases/download/v0.11.6/darwinia-x86_64-linux-gnu.tar.zst) |

### Docker

#### Pull with the Git Tag
```docker
docker pull {{ image_tag }}
```

#### Pull with the Git Commit SHA
```docker
docker pull {{ image_sha }}
```

## Notable Changes
- Substrate common changes: https://github.com/paritytech/polkadot/releases/tag/v0.9.8
- S2S audit: darwinia-network/darwinia-common#902
- Migrate staking to attribute macro: darwinia-network/darwinia-common#902
- Fix WASM execution: darwinia-network/darwinia-common#937 **!!IMPORTANT!!**
- Integrate the basic S2S bridge into Darwinia/Common runtimes: darwinia-network/darwinia#787

## Proposal Hashes

| Network  |          Proposal Hash          |
| :------: | :-----------------------------: |
| Darwinia | {{ darwinia_proposal_compact }} |
|   Crab   |   {{ crab_proposal_compact }}   |


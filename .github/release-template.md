Darwinia
===

## [0.11.7] - 2021-11-23
| Network  | Native Runtime | Upgrade Priority |
| :------: | :------------: | :--------------: |
| Darwinia |      1170      |       LOW        |
|   Crab   |      1170      |       LOW        |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                      Download                                                       |
| :---: | :----: | :---: | :---: | :-----------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.zst](https://github.com/darwinia-network/darwinia/releases/download/v0.11.7/darwinia-x86_64-linux-gnu.tar.zst) |
| Linux | x86_64 | 2.23  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia/releases/download/v0.11.7/darwinia-x86_64-linux-gnu.tar.bz2) |

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
- Substrate common changes: https://github.com/paritytech/polkadot/releases/tag/v0.9.9
- Integrate the wormhole pallets into Darwinia: darwinia-network/darwinia#812

## Proposal Hashes

| Network  |          Proposal Hash          |
| :------: | :-----------------------------: |
| Darwinia | {{ darwinia_proposal_compact }} |
|   Crab   |   {{ crab_proposal_compact }}   |


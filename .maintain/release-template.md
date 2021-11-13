Darwinia
===


## [0.90.x] - 2021-xx-xx
| Network  | Native Runtime | Upgrade Priority |
| :------: | :------------: | :--------------: |
| Darwinia |      115x      |       LOW        |
|   Crab   |      115x      |       LOW        |

### Resources

#### Pre-Built
|  OS   |  Arch  | Glibc | LLVM  |                                                      Download                                                       |
| :---: | :----: | :---: | :---: | :-----------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia/releases/download/v0.11.5/darwinia-x86_64-linux-gnu.tar.bz2) |

#### Docker

```docker
docker pull {{ image_tag }}
```

or

```docker
docker pull {{ image_sha }}
```

### Changes

> Substrate common changes: https://github.com/paritytech/polkadot/releases/tag/v0.9.5
- EVM related: darwinia-network/darwinia-common#817, darwinia-network/darwinia-common#837, darwinia-network/darwinia-common#867

### Proposal

| Network  | Proposal Hash                   |
| :------- | :------------------------------ |
| Darwinia | {{ darwinia_proposal_compact }} |
| Crab     | {{ crab_proposal_compact }}     |


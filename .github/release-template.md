Darwinia
===

## [0.11.7-1] - 2021-12-15
| Network  | Native Runtime | Upgrade Priority |
| :------: | :------------: | :--------------: |
| Darwinia |      1171      |       LOW        |
|   Crab   |      1171      |       LOW        |

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
- Enable wormhole in `OnDeliveryConfirmed`: darwinia-network/darwinia#819

## Proposal Hashes

| Network  |          Proposal Hash          |
| :------: | :-----------------------------: |
| Darwinia | {{ darwinia_proposal_compact }} |
|   Crab   |   {{ crab_proposal_compact }}   |


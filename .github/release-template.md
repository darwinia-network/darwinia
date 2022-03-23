Darwinia
===

## [0.12.0] - 2022-03-18

| Network  | Native Runtime | Upgrade Priority |
| :------: | :------------: | :--------------: |
| Darwinia |      1200      |       LOW       |
|   Crab   |      1200      |       LOW       |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                      Download                                                       |
| :---: | :----: | :---: | :---: | :-----------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.zst](https://github.com/darwinia-network/darwinia/releases/download/v0.12.0/darwinia-x86_64-linux-gnu.tar.zst) |
| Linux | x86_64 | 2.23  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia/releases/download/v0.12.0/darwinia-x86_64-linux-gnu.tar.bz2) |

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
- Substrate common changes: https://github.com/paritytech/polkadot/releases/tag/v0.9.11
- DVM changes: https://github.com/darwinia-network/darwinia/pull/836#issuecomment-1061327688

## Proposal Hashes

Compressed

| Network  |            Proposal Hash           |
| :------: | :--------------------------------: |
| Darwinia | {{ darwinia_proposal_compressed }} |
|   Crab   |   {{ crab_proposal_compressed }}   |

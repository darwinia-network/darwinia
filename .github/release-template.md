Darwinia
===

## [0.11.8] - 2021-12-15

> :warning: **This release introduces a new host function. Please upgrade your node prior to the next runtime upgrade of Crab or Darwinia in order for your node to continue syncing.**

| Network  | Native Runtime | Upgrade Priority |
| :------: | :------------: | :--------------: |
| Darwinia |      1180      |       HIGH       |
|   Crab   |      1180      |       HIGH       |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                      Download                                                       |
| :---: | :----: | :---: | :---: | :-----------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.zst](https://github.com/darwinia-network/darwinia/releases/download/v0.11.8/darwinia-x86_64-linux-gnu.tar.zst) |
| Linux | x86_64 | 2.23  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia/releases/download/v0.11.8/darwinia-x86_64-linux-gnu.tar.bz2) |

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
- Substrate common changes: https://github.com/paritytech/polkadot/releases/tag/v0.9.10

## Proposal Hashes

| Network  |          Proposal Hash          |
| :------: | :-----------------------------: |
| Darwinia | {{ darwinia_proposal_compact }} |
|   Crab   |   {{ crab_proposal_compact }}   |

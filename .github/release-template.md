Darwinia
===

‼️ MANDATORY: This is a MANDATORY priority release and you must upgrade as as soon as possible.

⚠️ It is critical that you update your client before the chain switches to the new runtimes. If you do not update your client before the new runtime is enacted, your node will stop handling parachains functionalities properly until you upgrade.
The changes motivating this priority level are:
- [EVM storage schema update](https://github.com/darwinia-network/darwinia/pull/864#issuecomment-1107740096)

## [0.12.1] - 2022-04-20

| Network  | Native Runtime | Upgrade Priority |
| :------: | :------------: | :--------------: |
| Darwinia |      1210      |       LOW        |
|   Crab   |      1210      |    MANDATORY     |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                      Download                                                       |
| :---: | :----: | :---: | :---: | :-----------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.zst](https://github.com/darwinia-network/darwinia/releases/download/v0.12.1/darwinia-x86_64-linux-gnu.tar.zst) |
| Linux | x86_64 | 2.23  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia/releases/download/v0.12.1/darwinia-x86_64-linux-gnu.tar.bz2) |

### Docker

#### Pull with the Git Tag
```docker
docker pull {{ image_tag }}
```

#### Pull with the Git Commit SHA
```docker
docker pull {{ image_sha }}
```

## Proposal Hashes

| Network  |           Proposal Hash            |
| :------: | :--------------------------------: |
| Darwinia | {{ darwinia_proposal_compressed }} |
|   Crab   |   {{ crab_proposal_compressed }}   |

## Changelog
[Project Release](https://github.com/orgs/darwinia-network/projects/8/views/1)

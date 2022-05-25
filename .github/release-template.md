Darwinia
===

## [{{ tag }}]

| Network  |         Native Runtime         | Upgrade Priority |
| :------: | :----------------------------: | :--------------: |
| Darwinia | {{ darwinia_runtime_version }} |       LOW        |
|   Crab   |   {{ crab_runtime_version }}   |       LOW        |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                       Download                                                        |
| :---: | :----: | :---: | :---: | :-------------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.zst](https://github.com/darwinia-network/darwinia/releases/download/{{ tag }}/darwinia-x86_64-linux-gnu.tar.zst) |
| Linux | x86_64 | 2.23  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia/releases/download/{{ tag }}/darwinia-x86_64-linux-gnu.tar.bz2) |

### Docker

#### Pull with the Git Tag
```docker
docker pull quay.io/darwinia-network/darwinia:{{ tag }}
```

#### Pull with the Git Commit SHA
```docker
docker pull quay.io/darwinia-network/darwinia:sha-{{ sha }}
```

## Proposal Hashes

| Network  |           Proposal Hash            |
| :------: | :--------------------------------: |
| Darwinia | {{ darwinia_proposal_compressed }} |
|   Crab   |   {{ crab_proposal_compressed }}   |

## Changelog
[Project Release](https://github.com/orgs/darwinia-network/projects/8/views/1)

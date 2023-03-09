Runtimes
===

| Network  |         Native Runtime         | Upgrade Priority |
| :------: | :----------------------------: | :--------------: |
| Darwinia | {{ darwinia_runtime_version }} |       LOW        |
|   Crab   |   {{ crab_runtime_version }}   |       LOW        |
| Pangolin | {{ pangolin_runtime_version }} |       LOW        |
| Pangoro  | {{ pangoro_runtime_version }}  |       LOW        |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM  |                                                       Download                                                        |
| :---: | :----: | :---: | :---: | :-------------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | 2.23  |  4.0  | [tar.zst](https://github.com/darwinia-network/darwinia/releases/download/{{ tag }}/darwinia-x86_64-linux-gnu.tar.zst) |
| Linux | x86_64 | 2.23  |  4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia/releases/download/{{ tag }}/darwinia-x86_64-linux-gnu.tar.bz2) |

### Docker

#### Pull with the Git Tag
```docker
docker pull ghcr.io/darwinia-network/darwinia:{{ tag }}
```

#### Pull with the Git Commit SHA
```docker
docker pull ghcr.io/darwinia-network/darwinia:sha-{{ sha }}
```

## Proposal Hashes

| Network  |            Proposal Hash             |           Spec Version           |
| :------: | :----------------------------------: | :------------------------------: |
| Darwinia | `{{ darwinia_proposal_compressed }}` | `{{ darwinia_runtime_version }}` |
|   Crab   |   `{{ crab_proposal_compressed }}`   |   `{{ crab_runtime_version }}`   |
| Pangolin | `{{ pangolin_proposal_compressed }}` | `{{ pangolin_runtime_version }}` |
| Pangoro  | `{{ pangoro_proposal_compressed }}`  | `{{ pangoro_runtime_version }}`  |

## Changelog
- #983
- #984
- #985
- #990
- #991
- #992
- #994
- #996

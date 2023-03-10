Darwinia
===

| Network  |         Native Runtime         | Upgrade Priority |
|:--------:|:------------------------------:|:----------------:|
| Darwinia | {{ darwinia_runtime_version }} |       LOW        |
|   Crab   |   {{ crab_runtime_version }}   |       LOW        |

## Resources

### Pre-built Binary
|  OS   |  Arch  | Glibc | LLVM |                                                       Download                                                        |
|:-----:|:------:|:-----:|:----:|:---------------------------------------------------------------------------------------------------------------------:|
| Linux | x86_64 | 2.23  | 4.0  | [tar.zst](https://github.com/darwinia-network/darwinia/releases/download/{{ tag }}/darwinia-x86_64-linux-gnu.tar.zst) |
| Linux | x86_64 | 2.23  | 4.0  | [tar.bz2](https://github.com/darwinia-network/darwinia/releases/download/{{ tag }}/darwinia-x86_64-linux-gnu.tar.bz2) |

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

|        -        |                Darwinia                |                Crab                |
|:---------------:|:--------------------------------------:|:----------------------------------:|
|  Proposal Hash  |  `{{ darwinia_proposal_compressed }}`  |  `{{ crab_proposal_compressed }}`  |
| Blake2 256 Hash | `{{ darwinia_blake2_256_compressed }}` | `{{ crab_blake2_256_compressed }}` |
|  Spec Version   |    `{{ darwinia_runtime_version }}`    |    `{{ crab_runtime_version }}`    |


## Changelog
[Darwinia 2.0](https://github.com/darwinia-network/darwinia/pull/969)


## Pangolin {{ pangolin_runtime_version }}
<h3 align="right">Upgrade Priority LOW :green_circle:</h3>
- Proposal Hash
  ```
  {{ pangolin_proposal_compressed }}
  ```
- Blake2 256 Hash
  ```
  {{ pangolin_blake2_256_compressed }}
  ```

## Pangoro {{ pangoro_runtime_version }}
<h3 align="right">Upgrade Priority LOW :green_circle:</h3>
- Proposal Hash
  ```
  {{ pangoro_proposal_compressed }}
  ```
- Blake2 256 Hash
  ```
  {{ pangoro_blake2_256_compressed }}
  ```

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

## Changelog

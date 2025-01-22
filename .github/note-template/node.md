
## Resources
### Pre-built Binary
|  OS   |  Arch  |                                                              Download                                                               |
| :---: | :----: | :---------------------------------------------------------------------------------------------------------------------------------: |
| Linux | x86_64 | [tar.zst](https://github.com/darwinia-network/darwinia/releases/download/{{ (ds "schema").tag }}/darwinia-x86_64-linux-gnu.tar.zst) |
| Linux | x86_64 | [tar.bz2](https://github.com/darwinia-network/darwinia/releases/download/{{ (ds "schema").tag }}/darwinia-x86_64-linux-gnu.tar.bz2) |

### Docker
#### Pull with the Git Tag
```docker
docker pull ghcr.io/darwinia-network/darwinia:{{ (ds "schema").tag }}
```
#### Pull with the Git Commit SHA
```docker
docker pull ghcr.io/darwinia-network/darwinia:sha-{{ (ds "schema").sha }}
```

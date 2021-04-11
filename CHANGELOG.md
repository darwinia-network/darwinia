

## [0.9.5-1] - 2021-02-21

Some breaking changes in `ethash`/`parity-scale-codec v2.0.0`. So downgrade it.


## [0.9.5] - 2021-02-21

### Darwinia 0.9.5
|  Chain   | Runtime Spec Version |
| :------: | :------------------: |
| Darwinia |          22          |
|   Crab   |          41          |

### Boot Flag

#### Darwinia
**Run with `--chain darwinia` or leave it empty to participate in Darwinia.**
**使用 `--chain darwinia` 或不填写，参与到 Darwinia 网络。**

#### Crab
**Run with `--chain crab` to participate in Crab.**
**使用 `--chain crab` ，参与到 Crab 网络。**

### Resource

#### Binary
|               |  Arch   | glibc (at least) | llvm (at least) | pre-build |
| :-----------: | :-----: | :--------------: | :-------------: | :-------: |
| General Linux | x86_64  |       2.17       |       3.8       |     ✔︎     |
|  RaspberryPi  | aarch64 |       2.23       |       3.8       |     ✔︎     |

#### Docker
```docker
docker pull quay.io/darwinia-network/darwinia:v0.9.5
```

### Change Log

#### Update(s)
- Substrate Updates [C 476](https://github.com/darwinia-network/darwinia-common/pull/476)
#### Fix(es)
- None
#### Migration(s)
- None

---

#### Darwinia Custom
##### Runtime
- None
##### Migration(s)
- None

---

#### Crab Custom
##### Runtime
- None
##### Migration(s)
- Remove Old Migration
- Fix Crab Staking Ledger




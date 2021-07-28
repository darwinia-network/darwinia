## [0.11.1] - 2021-07-28

### Upgrade Priority: MEDIUM
- With v0.11.1 client. Validators can specify a target gas price (e.g. `--target-gas-price=1000000000`) to adjust the EVM gas price. More detail can be found at [C 629](https://github.com/darwinia-network/darwinia-common/pull/629).

#### Upgrade Window Period - 升级窗口期
- None

#### Breaking Change(s)
- None

### Darwinia 0.11.1

|  Chain   | Runtime Spec Version |
| :------: | :------------------: |
| Darwinia |         1110         |
|   Crab   |         1110         |

### Boot Flag

#### Darwinia
**Run with `--chain darwinia` or leave it empty to participate in Darwinia.**
**使用 `--chain darwinia` 或不填写，参与到 Darwinia 网络。**

#### Crab
**Run with `--chain crab` to participate in Crab.**
**使用 `--chain crab` ，参与到 Crab 网络。**

### Resource

#### Binary
|               |  Arch  | glibc (at least) | llvm (at least) | pre-built |
| :-----------: | :----: | :--------------: | :-------------: | :-------: |
| General Linux | x86_64 |       2.23       |       4.0       |     ✔︎     |

#### Docker
```docker
docker pull quay.io/darwinia-network/darwinia:v0.11.1
```

### Change Log

#### Update(s)
- Substrate Updates [C 624](https://github.com/darwinia-network/darwinia-common/pull/624) [C 631](https://github.com/darwinia-network/darwinia-common/pull/631)
- Only For Darwinia Developers [C 21](https://github.com/darwinia-network/substrate-update-tracking/issues/21)
#### Fix(es)
- #685
#### Migration(s)
- https://github.com/paritytech/substrate/pull/8761 it's okay we never modify the `Recovery` in `construct_runtime!`
- https://github.com/paritytech/substrate/pull/8762 we don't have this pallet

---

#### Darwinia Custom
##### Runtime
- None
##### Migration(s)
- None

---

#### Crab Custom
##### Runtime
- DVM Update [C 629](https://github.com/darwinia-network/darwinia-common/pull/629)
##### Migration(s)
- None


## [0.11.0] - 2021-07-20

### IMPORTANT
**Please upgrade your node as SOON as possible!! Otherwise, your node might not sync the new blocks after enacting runtime.**
**请尽快升级您的节点！！否则在 Runtime 升级后，您的节点将有可能不会同步新的区块。**

#### Upgrade Window Period - 升级窗口期
  - Crab: Now ~ UTC 07-24-2021
  - Darwinia: Now ~ UTC 07-28-2021

#### Breaking Change(s)
- [Use host max log level when initializing the `RuntimeLogger`](https://github.com/paritytech/substrate/pull/8655), which introduced a new host function, which will be a breaking change once the corresponding runtime is enacted.

### Darwinia 0.11.0

|  Chain   | Runtime Spec Version |
| :------: | :------------------: |
| Darwinia |         1100         |
|   Crab   |         1100         |

### Boot Flag

#### Darwinia
**Run with `--chain darwinia` or leave it empty to participate in Darwinia.**
**使用 `--chain darwinia` 或不填写，参与到 Darwinia 网络。**

#### Crab
**Run with `--chain crab` to participate in Crab.**
**使用 `--chain crab` ，参与到 Crab 网络。**

### Resource

#### Binary
|               |  Arch  | glibc (at least) | llvm (at least) | pre-built |
| :-----------: | :----: | :--------------: | :-------------: | :-------: |
| General Linux | x86_64 |       2.23       |       4.0       |     ✔︎     |

#### Docker
```docker
docker pull quay.io/darwinia-network/darwinia:v0.11.0
```

### Change Log

#### Update(s)
- Substrate Updates [C 604](https://github.com/darwinia-network/darwinia-common/pull/604)
- Only For Darwinia Developers [C 19](https://github.com/darwinia-network/substrate-update-tracking/issues/19), [T 12](https://github.com/darwinia-network/substrate-update-tracking/issues/12), [T 15](https://github.com/darwinia-network/substrate-update-tracking/issues/15)
#### Fix(es)
- Prune `on-chain` MMR
  - https://github.com/darwinia-network/darwinia-common/pull/673
  - https://github.com/darwinia-network/darwinia-common/pull/689
  - #675
#### Migration(s)
- https://github.com/paritytech/substrate/pull/8620 pallet level migration
- https://github.com/paritytech/substrate/pull/8044 done
- https://github.com/paritytech/substrate/pull/7936 done (last time remaining migration)
- https://github.com/paritytech/substrate/pull/8414 pallet level migration
- https://github.com/paritytech/substrate/pull/8687 it's okay
- https://github.com/paritytech/substrate/pull/8663 it's okay we never modify the `Authorship` in `construct_runtime!`

---

#### Darwinia Custom
##### Runtime
- None
##### Migration(s)
- None

---

#### Crab Custom
##### Runtime
- Patch `evm-core` #662
##### Migration(s)
- Remove invalid schedule data #665


## [0.10.0] - 2021-04-24

### Darwinia 0.10.0
|  Chain   | Runtime Spec Version |
| :------: | :------------------: |
| Darwinia |          24          |
|   Crab   |          43          |

### Boot Flag

#### Darwinia
**Run with `--chain darwinia` or leave it empty to participate in Darwinia.**
**使用 `--chain darwinia` 或不填写，参与到 Darwinia 网络。**

#### Crab
**Run with `--chain crab` to participate in Crab.**
**使用 `--chain crab` ，参与到 Crab 网络。**

### Resource

#### Binary
|               |  Arch   | glibc (at least) | llvm (at least) | pre-built |
| :-----------: | :-----: | :--------------: | :-------------: | :-------: |
| General Linux | x86_64  |       2.17       |       3.8       |     ✔︎     |
|  RaspberryPi  | aarch64 |       2.23       |       3.8       |     ✔︎     |

#### Docker
```docker
docker pull quay.io/darwinia-network/darwinia:v0.10.0
```

### Change Log

#### Update(s)
- Substrate Updates [C 513](https://github.com/darwinia-network/darwinia-common/pull/513), [C 566](https://github.com/darwinia-network/darwinia-common/pull/566), [C 586](https://github.com/darwinia-network/darwinia-common/pull/586)
- Only For Darwinia Developers [T 5](https://github.com/darwinia-network/substrate-update-tracking/issues/5), [T 6](https://github.com/darwinia-network/substrate-update-tracking/issues/6), [T 7](https://github.com/darwinia-network/substrate-update-tracking/issues/7), [T 11](https://github.com/darwinia-network/substrate-update-tracking/issues/11)
#### Fix(es)
- None
#### Migration(s)
- https://github.com/paritytech/substrate/pull/8072 https://github.com/darwinia-network/darwinia/pull/641/commits/ee47efffe3e3086b7694034e888bfb90e54bdeeb
- https://github.com/paritytech/substrate/pull/8113 included in pallet
- https://github.com/paritytech/substrate/pull/8221 included in pellet

---

#### Darwinia Custom
##### Runtime
- None
##### Migration(s)
- None

---

#### Crab Custom
##### Runtime
- Remove Ropsten Bridge
- Set `RingExistentialDeposit` & `KtonExistentialDeposit` to Zero
- Add DVM
##### Migration(s)
- Move `ethfe` account's balances to multisig account #633
- Remove Ropsten Bridge #633


## [0.9.6] - 2021-04-12

### Darwinia 0.9.6
|  Chain   | Runtime Spec Version |
| :------: | :------------------: |
| Darwinia |          23          |
|   Crab   |          42          |

### Boot Flag

#### Darwinia
**Run with `--chain darwinia` or leave it empty to participate in Darwinia.**
**使用 `--chain darwinia` 或不填写，参与到 Darwinia 网络。**

#### Crab
**Run with `--chain crab` to participate in Crab.**
**使用 `--chain crab` ，参与到 Crab 网络。**

### Resource

#### Binary
|               |  Arch   | glibc (at least) | llvm (at least) | pre-built |
| :-----------: | :-----: | :--------------: | :-------------: | :-------: |
| General Linux | x86_64  |       2.17       |       3.8       |     ✔︎     |
|  RaspberryPi  | aarch64 |       2.23       |       3.8       |     ✔︎     |

#### Docker
```docker
docker pull quay.io/darwinia-network/darwinia:v0.9.6
```

### Change Log

#### Update(s)
- Substrate Updates [C 468](https://github.com/darwinia-network/darwinia-common/pull/468), [C 499](https://github.com/darwinia-network/darwinia-common/pull/499)
- Only For Darwinia Developers [T 4](https://github.com/darwinia-network/substrate-update-tracking/issues/4)
#### Fix(es)
- None
#### Migration(s)
- `darwinia_elections_phragmen::migrations_2_0_0` [C 525](https://github.com/darwinia-network/darwinia-common/pull/525)

---

#### Darwinia Custom
##### Runtime
- None
##### Migration(s)
- None

---

#### Crab Custom
##### Runtime
- Disable Ropsten Bridge
##### Migration(s)
- Move `ethbk` account's balances to multisig account #633


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
|               |  Arch   | glibc (at least) | llvm (at least) | pre-built |
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

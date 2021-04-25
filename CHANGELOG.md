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
|               |  Arch   | glibc (at least) | llvm (at least) | pre-build |
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
|               |  Arch   | glibc (at least) | llvm (at least) | pre-build |
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




[![CI](https://travis-ci.org/darwinia-network/darwinia.svg)](https://travis-ci.org/darwinia-network/darwinia])
[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

![Logo](https://github.com/darwinia-network/rfcs/raw/master/logo/darwinia.png)

# Darwinia
Implementation of a https://darwinia.network node in **Rust** based on the **Substrate** framework.

This repository contains runtimes for the **[Darwinia](https://darwinia.network)** and **[Crab](https://crab.network)** networks.

## Resources

### Documents
- [Darwinia Network Docs](https://docs.darwinia.network)
- [Crab Network Docs](https://docs.crab.network)

### Technical Support
- Telegram
	- [Official Technical Group](https://t.me/DarwiniaDev)
- Matrix
	- [Official Technical Room](https://matrix.to/#/#darwinia:matrix.org)
- Slack
	> Technical members are more active on this place. If you expect a quick response, here you go.
	- PM [Support](mailto:support@darwinia.network) or [Xavier](mailto:xavier.lau@itering.com)

## Installation
- Downloading pre-built binary from **[releases](https://github.com/darwinia-network/darwinia/releases)** page.
- Using the docker image on **[releases](https://github.com/darwinia-network/darwinia/releases)** page.
- Building from source follow this **[guide](#build-from-source)**.

## Building from Source
> Make sure that you have the dependencies. Follow [substrate-getting-started](https://substrate.dev/docs/en/knowledgebase/getting-started).

### Installing via Cargo
```sh
cargo install --git https://github.com/darwinia-network/darwinia --tag <version> --locked
```

### Building via Source
```sh
# with github-cli
gh repo clone darwinia-network/darwinia
# with git
git clone https://github.com/darwinia-network/darwinia.git
git checkout <version>
cargo build --release --locked
```

## Networks
This repository supports runtimes for Darwinia and Crab.

### Connecting to Darwinia Mainnet
Connecting to the global Darwinia network by running:
```sh
./darwinia --chain darwinia
```
You can see your node on [telemetry](https://telemetry.polkadot.io/#list/0x729cb8f2cf428adcf81fe69610edda32c5711b2ff17de747e8604a3587021db8) (set a custom name with --name "my custom name").

### Connecting to Crab Canary Network
Connecting to the global Crab Canary Network by running:
```sh
./darwinia --chain crab
```
You can see your node on [telemetry](https://telemetry.polkadot.io/#list/0x34f61bfda344b3fad3c3e38832a91448b3c613b199eb23e5110a635d71c13c65) (set a custom name with --name "my custom name").

## Contributing

### Roadmap
[Roadmap](docs/ROADMAP.md)

### Contributing Guidelines
[Contributing Guidelines](docs/CONTRIBUTING.adoc)

### Contributor Code of Conduct
[Code of Conduct](docs/CODE_OF_CONDUCT.md)

### Security
[Security](docs/SECURITY.md)

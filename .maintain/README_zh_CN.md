Darwinia 维护指北
===

## 发布策略

Darwinia 项目发布交由 Github Actions 自动化处理. 其中自动发布的内容包括

- Darwinia 二进制分发文件; 目前支持的 CPU 架构包含
	- x86_64
- Runtime Webassembly 文件 (wasm); 支持的 Runtime 包含
	- [Darwinia](../runtime/darwinia)
	- [Crab](../runtime/crab)

## 如何使用

当下发布策略, 完全采用 Github Actions 自动发布, 共有两种发布模式.


### 开发模式

配置文件: [.github/workflows/ci.yml](../.github/workflows/ci.yml)

在开发模式中, 任何有关 master 分支的代码提交事件 (包括 pull request), 都会对最新提交的代码进行测试工作

```bash
cargo test
```

所有在项目中添加的测试代码都将会使用 `cargo` 来进行验证.

### 里程碑模式

配置文件:

- [.github/workflows/release.yml](../.github/workflows/release.yml)
- [.github/workflows/rustdocs.yml](../.github/workflows/rustdocs.yml)

里程碑模式对应的就是 `git tag` 指令, 发布新的版本. 为了能快速正确的发布新的版本, 需要知道下方说明的相关事项:

1. 标签名称规范
   在目前的发布策略中, 要求发布的标签名称以 `v` 开头, 例如当前的最新版本 `0.9.5-1` 需要使用 `v0.9.5-1`

2. 修改 [release-template.md](./release-template.md)
   Github release page 页面所显示的内容是根据此模板而生成的. 如果需要请编辑此模板.

## subwasm

Darwinia 直接采用 subwasm 来进行 runtime wasm proposal hash 计算, 采用与 srtool 相同的环境, 在计算完成后, 会输出变量到 release page 模板中, 视需要输出到页面, 同时也会将计算出的 json 存储为文件上传到 release page.

### 验证

如果想要自行验证 wasm hash, 可以通过 polkadot 钱包在上传 wasm 后会得到一个 proposal hash, 然后与 CI 发布的进行对比.


### srtool

srtool 目前已不适用于 Darwinia, 因为 srtool 目前所发布的镜像命名规则变更, 无法准确定位到 Darwinia 所需要的版本. 因此目前以不建议在 Darwinia 上使用.




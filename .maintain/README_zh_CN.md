Darwinia 维护指北
===

## 发布策略

自 `0.9.5-1` 版本之后, Darwinia 项目发布将采用新的策略, 全交由 Github Actions 自动化处理. 其中自动发布的内容包括

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

配置文件: [.github/workflows/release.yml](../.github/workflows/release.yml)

里程碑模式对应的就是 `git tag` 指令, 发布新的版本. 为了能快速正确的发布新的版本, 需要知道下方说明的相关事项:

1. 标签名称规范
   在目前的发布策略中, 要求发布的标签名称以 `v` 开头, 例如当前的最新版本 `0.9.5-1` 需要使用 `v0.9.5-1`

2. [CHANGELOG.md](../CHANGELOG.md) 编写
   任何一次版本发布都需要编写 [CHANGELOG.md](../CHANGELOG.md) 文件; 建议遵循 [keep a changelog](https://keepachangelog.com/zh-CN/1.0.0/) 规范, 根据实际情况考量. 但是需要特别注意的是, 每次新版本发布, 新添加的版本号需要和 `git tag` 所创建的版本号一致, 不同添加前缀 `v`, 例如 `git tag v0.9.5-1` 那么需要在 [CHANGELOG.md](../CHANGELOG.md) 中添加 `## 0.9.5-1` 开始的块; 部署脚本将会提取此块放入到 Github Release 页面.

> 注意: CHANGELOG.md 的修改是必须的, 而且新增的内容要和发布的版本号一致, 否则部署将无法通过.

因此, 一个完整的里程碑发布应该遵循一下步骤 (假设以发布 `0.9.6` 版本为例):


1. 添加更新内容至 [CHANGELOG.md](../CHANGELOG.md),
   ```markdown
   ## [0.9.6] - 2021-04-01

   ### Added
   - Add a new runtime

   ### Changed
   - fix some bugs

   ## [0.9.5-1] - 2020-03-20
   ...
   ```

2. 推送至 master 分支
   ```bash
   git add .
   git commit -m "Release v0.9.6"
   git push origin master
   ```

3. 创建 `tag`
   ```bash
   git tag v0.9.6
   git push v0.9.6
   ```

4. 等待自动发布

## 缓存

为了加快整个编译周期, 减少编译的时间, 目前的编译环境采用了 [Github Actions Cache](https://docs.github.com/cn/actions/guides/caching-dependencies-to-speed-up-workflows) 来缓存部分内容. 让编译提速.

然而 Cache 有[容量限制](https://docs.github.com/cn/actions/guides/caching-dependencies-to-speed-up-workflows#usage-limits-and-eviction-policy):

> GitHub 将删除 7 天内未被访问的任何缓存条目。 可以存储的缓存数没有限制，但存储库中所有缓存的总大小限制为 5 GB。 如果超过此限制，GitHub 将保存缓存，但会开始收回缓存，直到总大小小于 5 GB。

目前缓存的策略并非是 Cargo 所下载的依赖库, 而是将编译过程中的 `target` 目录进行了缓存, 因为就实际情况而言, 在 Github Actions 的编译环境中, 网络并不是一个非常慢的问题, 主要拖慢速度的过程还是在编译, 因此将 `target` 目录进行缓存, 将会大大减少编译的时间.

但是目前缓存策略还存在一下相关问题:

- 已编译依赖的缓存命中
  通常情况 Cargo 编译, 在相同的环境除非是版本发生变动, 已编译过的依赖将不会重新编译; 不过在 darwinia 项目中很多依赖并非是直接使用的 [crates.io](https://crates.io/) 发布的库, 而是直接依赖的 Github 仓库 (例如 [substrate](https://github.com/darwinia-network/substrate)), 这些仓库在 Cargo 的编译过程中, 都不会命中以前以编译过的文件, 而是选择重新编译; 因此这部分库仍然需要等待其编译.
- 容量
  因为 cache 有容量限制, darwinia 编译通常不会超过这个限制, 然而在日积月累的使用中, 代码的增加, 以及依赖的版本升级等等, 最终可能会超过这个限制; 不过不用太过担心, 如果 Cache 的删除策略启动, 部分已缓存的已编译依赖发生损毁等相关问题, Cargo 通常会重新编译依赖, 然后再次写入到 cache 中.

加入 cache 后的效果是非常明显的, 一次编译的时间从之前的 50 分钟缩短到 20 分钟. 由于里程碑模式的变动, 需要操作的事项变多, 时间将会在 1 小时左右.

## srtool

需要特别说明的是, 在目前的发布方式中链上升级文件 wasm 使用 [srtool](https://github.com/paritytech/srtool) 进行编译, 在不同的机器/系统中所编译出来 wasm 文件会有些许不同, srtool 的目的就是为了统一编译环境, 使所有 wasm 是在相同的环境下进行编译的.

srtool 被发布在 Docker hub [paritytech/srtool](https://hub.docker.com/r/paritytech/srtool) 镜像中, 这个镜像是 Parity 根据 [chevdor/srtool](https://gitlab.com/chevdor/srtool) 所发布的. Parity 所维护的镜像相对更新并不及时, 并且只有一个可用的镜像标签, 目前编译使用的是 Parity 发布的镜像.


### 使用 srtool


关于 srtool 的使用方式, 这里进行简单的说明

> 更详细的使用说明可以参考 [paritytech/srtool/README.adoc](https://github.com/paritytech/srtool/blob/master/README.adoc), 由于 Parity 更新的不及时, 这里文档所列出的部分使用方式并不能正常工作, 因为 Docke hub 中没有对应的镜像, 因此也可以去看 [chevdor/srtool/README.adoc](https://gitlab.com/chevdor/srtool/-/blob/master/README.adoc)


首先, srtool 支持的命令包括

- `help`: 查看帮助
- `version`: srtool 版本
- `info`: 在运行构建之前查看一些系统信息
- `build`: 进行 runtime 编译


由于 srtool 发布了 docker 镜像, 可以利用 docker 的运作模式配置 bash 等相关特性来构建一个可执行的脚本文件

例如官方所发布的一个简单的别名脚本

```bash
export RUSTC_VERSION=nightly-2021-03-15;
export PACKAGE=kusama-runtime;
alias srtool='docker run --rm -it -e PACKAGE=$PACKAGE -v $PWD:/build -v $TMPDIR/cargo:/cargo-home chevdor/srtool:$RUSTC_VERSION'
```

这里的含义是, 将目前所在路径挂载到 docker 容器中, 并使用指定的 rust 版本来对当前 runtime 进行编译. 其中 `PACKAGE` 就是需要编译的 runtime 名称


编译使用

```bash
srtool build
```

进行编译后的输出后内容类似于下方


```text
🧰 Substrate Runtime Toolbox - srtool v0.9.6 🧰
          - by Chevdor -
🏗  Building polkadot-runtime as release using rustc 1.49.0-nightly (fd542592f 2020-10-26)
⏳ That can take a little while, be patient... subsequent builds will be faster.
Since you have to wait a little, you may want to learn more about Substrate runtimes:
https://substrate.dev/docs/en/#architecture

    Finished release [optimized] target(s) in 37.43s

real  0m37.931s
user  0m1.560s
sys 0m3.220s
✨ Your Substrate WASM Runtime is ready! ✨
Summary:
Used rustc nightly-2021-03-15 (4560ea788 2019-11-04)
Wasm     : ./[some path]/polkadot_runtime.compact.wasm
Content  : 0x0061736d0100000001a4022b60037f7f...3435663020323031392d31322d303429
Size     : 1.1M
Proposal : 0x5931690e71e9d3d9f04a43d8c15e45e0968e563858dd87ad6485b2368a286a8f
SHA256   : 0xd5930520676994fc55a29c547f0159ea860cb46edd710a5be35e62565af1ad8b
```

同时支持以 json 格式输出

```bash
srtool build --json
```


```text
{
  "gen": "srtool",
  "rustc": "rustc 1.41.0-nightly (ae1b871cc 2019-12-06)",
  "wasm": "./target/srtool/release/wbuild/kusama-runtime/kusama_runtime.compact.wasm",
  "size": "1205052",
  "pkg": "kusama-runtime",
  "prop": "0x5931690e71e9d3d9f04a43d8c15e45e0968e563858dd87ad6485b2368a286a8f",
  "sha256": "0xd93126c814f8366b651e425e34390212a98f8e77a8b73f9e1d2b07fc229a25f1",
  "tmsp": "2020-01-14T10:15:28Z"
}
```

这里需要注意的是 `Proposal` 字段, 这个值应该要和 apps 中发布时的值相同


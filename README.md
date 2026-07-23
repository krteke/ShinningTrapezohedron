# ShinningTrapezohedron

Milk-V Duo S 的 Rust 守护进程与 SvelteKit 管理界面。

## 构建

首次构建前安装前端依赖：

```bash
pnpm --dir frontend install --ignore-workspace --frozen-lockfile
```

之后执行 Rust 构建。`build.rs` 会直接调用当前环境中的 pnpm，生成前端静态文件并嵌入 Rust 二进制：

```bash
cargo build --release
```

### RISC-V musl 静态构建

首先安装 RISC-V 目标工具链：

```bash
rustup target add riscv64gc-unknown-linux-musl
```

然后执行构建：

```bash
cargo build --release --target riscv64gc-unknown-linux-musl
```

## 运行

先复制并编辑配置文件，所有运行参数都从该文件读取：

```bash
cp config.example.toml config.toml
cargo run -- config.toml
```

- `GET /api/health` 返回进程健康状态。
- `GET /api/status` 返回状态采集器发布的最新不可变快照。
- `GET /api/config` 返回当前配置快照和各字段的生效时机。
- `PUT /api/config` 接收完整候选配置；JSON 内层字段名与 TOML 保持一致。
- 未匹配的页面路径返回嵌入的前端入口。
- 未匹配的 `/api/*` 路径始终返回 JSON 404，不会落入前端页面。

状态通过 `tokio::sync::watch` 发布。Linux 采集器按配置文件中的间隔读取 `/proc`，成功后整体替换快照；采集失败时继续保留上一次完整状态。

运行期配置由单一 actor 串行处理。候选配置会先验证，再通过同目录临时文件原子替换旧配置，持久化成功后才发布新的 `watch` 快照。状态采样周期变更可热生效；Web 监听地址和日志输出配置目前仍需重启进程才能完整生效。

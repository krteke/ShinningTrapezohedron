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

默认监听 `0.0.0.0:3000`，可以通过环境变量覆盖：

```bash
SHINNING_LISTEN_ADDR=127.0.0.1:3000 cargo run
```

- `GET /api/health` 返回进程健康状态。
- `GET /api/status` 返回状态采集器发布的最新不可变快照。
- 未匹配的页面路径返回嵌入的前端入口。
- 未匹配的 `/api/*` 路径始终返回 JSON 404，不会落入前端页面。

状态通过 `tokio::sync::watch` 发布。Linux 采集器每两秒读取一次 `/proc`，成功后整体替换快照；采集失败时继续保留上一次完整状态。

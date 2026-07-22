# ShinningTrapezohedron

Milk-V Duo S 的 Rust 守护进程与 SvelteKit 管理界面。

## 构建

首次构建前安装前端依赖：

```bash
pnpm --dir frontend install --ignore-workspace --frozen-lockfile
```

之后执行 Rust 构建。`build.rs` 会通过 Corepack 调用固定版本的 pnpm，生成前端静态文件并嵌入 Rust 二进制：

```bash
cargo build --release
```

## 运行

默认监听 `0.0.0.0:3000`，可以通过环境变量覆盖：

```bash
SHINNING_LISTEN_ADDR=127.0.0.1:3000 cargo run
```

- `GET /api/health` 返回进程健康状态。
- 未匹配的页面路径返回嵌入的前端入口。
- 未匹配的 `/api/*` 路径始终返回 JSON 404，不会落入前端页面。

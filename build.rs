use std::{env, path::PathBuf, process::Command};

fn main() {
    for path in [
        "frontend/src",
        "frontend/static",
        "frontend/package.json",
        "frontend/pnpm-lock.yaml",
        "frontend/vite.config.ts",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }

    let frontend_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("无法读取 Cargo 工程目录"))
            .join("frontend");
    let status = Command::new("pnpm")
        .args(["--ignore-workspace", "run", "build"])
        .current_dir(frontend_dir)
        .status()
        .expect("无法启动 pnpm，请先安装 Node.js 和 pnpm");

    assert!(status.success(), "前端生产构建失败");
}

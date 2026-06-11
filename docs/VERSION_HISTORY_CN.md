# Nina Rust 版本历史

## v1.0.0 MVP

定位：基础可运行版本，证明 Rust CLI 可以通过 Ableton Remote Script 控制 Ableton Live。

### 已实现功能

- Rust CLI 与 `NinaRustBridge` 的 localhost TCP/JSON 通信。
- `live health`：检查 Remote Script 是否在线。
- `live snapshot`：读取 tempo、拍号、播放状态、track/scene 摘要。
- `live tempo set`：设置 Ableton tempo。
- `live transport play/stop`：控制播放和停止。
- `clip create`：在指定 MIDI track 的 Arrangement 里按 bar 区间创建 MIDI clip。
- `clip write-midi`：读取 MIDI JSON，校验后写入 Ableton MIDI clip。
- `browser scan`：扫描 Ableton browser 根分类的第一层 items。
- Remote Script 只绑定 `127.0.0.1`，不暴露局域网端口。
- 提供 Rust contract tests 与 Python Remote Script tests。

### 验证命令

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
python3 -m unittest tests.remote_script.test_nina_rust_bridge -v
```

### 当前限制

- 还不是 3000 行以上的最终课程完整版。
- 还未改成 Rust workspace。
- 暂未接入大模型 API。
- 暂未实现 Rust 自动旋律/节奏生成 engine。
- browser 只做根级扫描，不做递归索引和语义搜索。

## 后续规划

### v2.0 协作增强层

适合作为组员功能分支：

- MIDI import/export。
- browser index/search v2。
- Rust melody/rhythm generator。
- Rust 并发 watch mode。

### v3.0 课程最终版

目标：

- Rust 代码行数达到 3000 行以上。
- workspace 拆分为 CLI / protocol / client / engine / test-support。
- 补充模块内 unit tests。
- 补充课程要求映射文档。
- 通过 `cargo fmt`、`cargo clippy`、`cargo test`。

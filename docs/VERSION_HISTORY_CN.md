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

### v2.0 MIDI Pipeline

定位：Rust 不负责作曲生成，而是负责接收、校验、预览、转换和写入外部生成的 MIDI JSON。

当前 `feature/v2-midi-pipeline` 已加入：

- `track create-midi`：在 Ableton 中创建 MIDI track。
- `midi validate`：校验外部 MIDI JSON。
- `midi preview`：预览 note 数量、pitch 范围、velocity 范围和 clip 长度。
- `midi transpose`：整体转调，输出新的 MIDI JSON。
- `midi quantize`：按网格量化 start/duration，输出新的 MIDI JSON。

后续适合作为组员功能分支：

- MIDI import/export。
- browser index/search v2。
- Rust 并发 watch mode。

### v3.0 MIDI Toolkit

定位：本地 MIDI 文件工具层，脱离 Ableton 也可以独立运行。

当前 `feature/v3-midi-toolkit` 已加入：

- `midi export`：把 Nina MIDI JSON 导出为标准 `.mid` 文件。
- `midi import`：把本地 `.mid` 文件导入为 Nina MIDI JSON。
- `midi import` 不指定 `--output` 时，会在输入 MIDI 的同目录生成同名 `.json`。

示例：

```bash
cargo run -- midi import --input /path/to/demo.mid --track 2 --start-bar 1
```

默认输出：

```text
/path/to/demo.json
```

### v3.0 课程最终版

目标：

- Rust 代码行数达到 3000 行以上。
- workspace 拆分为 CLI / protocol / client / engine / test-support。
- 补充模块内 unit tests。
- 补充课程要求映射文档。
- 通过 `cargo fmt`、`cargo clippy`、`cargo test`。

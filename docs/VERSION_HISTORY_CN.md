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

### v4.0 Browser Index / Sound Library Layer

定位：音色库索引层。它不负责作曲生成，而是把 Ableton Browser 的一级扫描结果保存成本地 JSON，随后支持离线搜索和随机选择。

当前 `feature/v4-browser-index` 已加入：

- `browser index`：调用 Remote Script 扫描 Ableton Browser 根分类，并保存成本地索引 JSON。
- `browser search`：读取本地索引 JSON，按 query token 搜索 name、path、tags。
- `browser random`：读取本地索引 JSON，按 seed 做可复现随机选择。
- `src/browser/` 分层：`index.rs`、`search.rs`、`random.rs`。
- `tests/browser_index_contract.rs`：覆盖索引建立、读写、搜索、过滤和随机选择。
- `live watch`：持续轮询 Ableton snapshot，输出状态变化事件。
- `live diff`：对比两个 snapshot JSON 文件，识别 tempo、播放状态、轨道变化。
- `src/live/` 分层：`event.rs`、`diff.rs`、`recorder.rs`、`watcher.rs`。
- `tests/live_watch_contract.rs`：覆盖 snapshot diff、JSONL recorder、watch loop。
- `device scan`：扫描指定 track 上挂载的 MIDI effect、Instrument/Rack、Audio effect。
- Device Chain Inspector 默认返回 device 摘要、参数数量和 Rack chains 摘要。
- `device scan --include-parameters` 会展开完整参数列表和显示值，但当前只读，不修改参数。
- `drum scan`：扫描 Drum Rack 的 pad/note 映射，识别 kick、snare、hat 等角色，供后续大模型编鼓使用。

示例：

```bash
cargo run -- browser index --root sounds --limit 200 --output .nina/sounds_index.json
cargo run -- browser search --index .nina/sounds_index.json --query "cold pad" --limit 10
cargo run -- browser random --index .nina/sounds_index.json --seed 42 --loadable-only
cargo run -- live watch --interval-ms 1000 --count 10
cargo run -- live diff --before .nina/snapshot_before.json --after .nina/snapshot_after.json
cargo run -- device scan --track 2
cargo run -- device scan --track 2 --include-parameters
cargo run -- drum scan --track 2
```

### v5.0 Audio Layer

定位：把 Nina 从 MIDI 控制扩展到音频轨和音频片段管理，同时保留清晰的 Ableton API 能力边界。

当前 `feature/v5-audio-layer` 已加入：

- `track create-audio`：创建 Ableton audio track。
- `audio import`：把本地音频文件导入到指定 audio track 的 Arrangement 指定 bar 位置。
- `audio effects`：扫描 audio track 的效果链，不默认展开参数值。
- `audio clips`：扫描 audio track 上的 Arrangement audio clips，返回名称、文件路径、开始时间和长度。
- `audio context`：合并 audio clips 与 audio effects，输出后续 agent/MCP 可消费的上下文 JSON。
- `audio analyze-file`：本地分析音频文件路径、文件名、扩展名、格式猜测和文件大小，不连接 Ableton。
- `audio to-midi`：调用 Ableton `Live.Conversions.audio_to_midi_clip`，实验性支持 drums / melody / harmony 三种转换模式。
- `media plan`：为外部媒体 provider 生成 manifest，目前支持 `stem-split + dry-run`，用于约定 vocals/drums/bass/other 等 stem 输出。
- `media status`：读取 manifest 并检查外部媒体 provider 的输出文件是否已经生成。
- `src/audio/` 分层：本地音频文件校验、bar 到 Ableton beat time 的转换。
- `src/media/` 分层：`MediaProvider` trait、media request/manifest/verification、manifest save/load。
- 明确不伪装实现不稳定的 track audio export/render；stem separation 暂作为后续 provider 层规划。

示例：

```bash
cargo run -- track create-audio --name "Printed Stems"
cargo run -- audio import --track 3 --file /absolute/path/to/loop.wav --bar 5 --name "Loop Print"
cargo run -- audio effects --track 3
cargo run -- audio clips --track 3
cargo run -- audio context --track 3
cargo run -- audio analyze-file --file /absolute/path/to/loop.wav
cargo run -- audio to-midi --track 3 --clip-index 1 --mode drums
cargo run -- media plan --lane stem-split --provider dry-run --input /absolute/path/song.wav --output-dir /absolute/path/stems --stems vocals,drums,bass,other
cargo run -- media status --manifest /absolute/path/stems/nina_media_manifest.json
```

### v3.0 课程最终版

目标：

- Rust 代码行数达到 3000 行以上。
- workspace 拆分为 CLI / protocol / client / engine / test-support。
- 补充模块内 unit tests。
- 补充课程要求映射文档。
- 通过 `cargo fmt`、`cargo clippy`、`cargo test`。

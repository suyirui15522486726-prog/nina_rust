# Nina Rust V5 Audio Layer

V5 的目标是把 Nina 从“只会操作 MIDI”扩展到“能管理 Ableton 音频轨和音频片段”。本版本坚持一个边界：Rust CLI 负责本地校验、协议建模和命令编排；Ableton Remote Script 只负责调用 Live Object Model 或 Live 内部已存在的转换能力。

## 已完成功能

### 1. 创建 Audio Track

命令：

```bash
cargo run -- track create-audio --name "Printed Stems"
cargo run -- track create-audio --position 3 --name "Vocal Print"
```

说明：

- `--position` 是用户视角的轨道编号，从 1 开始。
- Rust 会转换成 Remote Script 使用的 0-based index。
- Remote Script 调用 Ableton 的 `song.create_audio_track(index)`。

### 2. 导入本地音频文件到 Arrangement

命令：

```bash
cargo run -- audio import \
  --track 3 \
  --file /absolute/path/to/loop.wav \
  --bar 5 \
  --name "Loop Print"
```

说明：

- Rust 会检查文件是否存在，并把路径 canonicalize 成绝对路径。
- Rust 会读取当前 Live set 拍号，把 `--bar` 转换成 Ableton 的 beat time。
- Remote Script 调用 `track.create_audio_clip(file_path, destination_time)`。
- 目标轨道必须是 audio track；如果传入 MIDI track，会返回明确错误。

### 3. 扫描 Audio Track 效果链

命令：

```bash
cargo run -- audio effects --track 3
```

说明：

- 只返回设备链结构，不返回参数值。
- 这个命令适合给后续 agent 或大模型看“当前音频轨挂了哪些效果器”。
- 参数扫描仍然由已有的 `device scan --include-parameters` 负责，避免默认输出过长。

### 4. Audio Context Layer

命令：

```bash
cargo run -- audio clips --track 3
cargo run -- audio context --track 3
cargo run -- audio analyze-file --file /absolute/path/to/loop.wav
```

说明：

- `audio clips` 扫描指定 audio track 的 Arrangement audio clips，返回 clip 名称、文件路径、开始时间和长度。
- `audio context` 把 audio clips 和效果链合并成一个 agent-readable JSON。
- `audio analyze-file` 是本地 Rust 功能，不连接 Ableton；它检查文件并返回文件名、扩展名、格式猜测和字节大小。
- 这层是 V6 MCP / LangGraph agent 的状态输入基础：agent 先读取 context，再决定是否导入、转 MIDI、询问用户或调用后续 stem provider。

示例输出形状：

```json
{
  "track": {
    "index": 2,
    "name": "Vocal Print",
    "has_midi_input": false,
    "has_audio_input": true
  },
  "clip_count": 1,
  "clips": [
    {
      "index": 0,
      "name": "verse.wav",
      "file_path": "/path/to/verse.wav",
      "start_time": 16.0,
      "length": 32.0,
      "is_audio_clip": true
    }
  ],
  "effect_count": 2,
  "effects": [
    {
      "index": 0,
      "name": "EQ Eight",
      "class_name": "Eq8",
      "role": "audio_effect",
      "is_rack": false,
      "parameter_count": 8,
      "chain_count": 0,
      "parameters": [],
      "chains": []
    }
  ]
}
```

### 5. Audio Clip 转 MIDI

命令：

```bash
cargo run -- audio to-midi --track 3 --clip-index 1 --mode drums
cargo run -- audio to-midi --track 3 --clip-index 1 --mode melody
cargo run -- audio to-midi --track 3 --clip-index 1 --mode harmony
```

说明：

- `--clip-index` 是用户视角的音频 clip 编号，从 1 开始。
- Remote Script 使用 Ableton 内部 `Live.Conversions.audio_to_midi_clip`。
- 支持 `drums`、`melody`、`harmony` 三种模式。
- 这是实验性能力：Ableton 不把它作为稳定公开 SDK 承诺，因此 Nina 会做运行时检查，并在不可用时返回错误。

## 不做伪功能

### 不直接实现 Track Export Audio

Ableton 的 Export Audio/Video 是 GUI 菜单工作流，Remote Script/LOM 没有稳定公开的 render/export API。本项目不会伪装成已经支持一键导出 audio track。

如果后续要做，建议作为 human-in-the-loop 流程：

1. Nina 创建/整理轨道。
2. 用户在 Ableton 中执行导出或 freeze/flatten。
3. Nina 扫描生成的音频文件并导入回工程。

### 不把 Logic Pro 逆向分轨做成核心依赖

Logic Pro Stem Splitter 没有公开第三方 CLI/SDK。公开课程仓库不应分发逆向模型、私有权重或绕过式调用。后续如果做 stem separation，建议做 provider 抽象：

```text
Nina stem provider
├─ open-source ONNX / Demucs provider
├─ external CLI adapter
└─ experimental local-only provider
```

这样项目架构更干净，也更适合公开仓库和课程评分。

## Media Provider Layer

V5 继续加入了一个轻量的 media provider 协议层，详见 `docs/V5_MEDIA_PROVIDER_CN.md`。

它目前提供：

```bash
cargo run -- media plan \
  --lane stem-split \
  --provider dry-run \
  --input /absolute/path/song.wav \
  --output-dir /absolute/path/stems \
  --stems vocals,drums,bass,other

cargo run -- media status \
  --manifest /absolute/path/stems/nina_media_manifest.json
```

这层不是直接分轨，而是生成和验证外部媒体任务的 manifest。未来如果 audio 侧接 Suno、Demucs 或本地分轨模型，TS/JS agent 可以先调用 `media plan`，再把任务交给对应 provider，最后用 `media status` 确认输出文件是否已就绪。

对应关系：

```text
MIDI lane
  GPT / 普通 LLM -> Nina MIDI JSON -> Rust validate/export/write-midi

Audio lane
  Suno / 分轨模型 / 外部 CLI -> audio files -> Rust manifest/status/import/context
```

## 架构体现

```text
src/audio/
  本地音频路径校验、bar 到 beat 的转换、本地文件分析

src/media/
  外部媒体 provider trait、任务 manifest、输出文件验证

src/protocol.rs
  Audio command payload/result/context 的 serde 契约

src/client.rs
  AbletonClient 的 typed audio methods

src/cli.rs
  用户命令、1-based 到 0-based 转换、本地文件校验

remote_scripts/NinaRustBridge/__init__.py
  调用 Ableton Live API 的薄桥接层

tests/
  Rust 协议测试、CLI 帮助测试、本地 audio contract 测试

tests/remote_script/
  Fake Live 环境下的 Remote Script 行为测试
```

V5 不是简单堆命令，而是把 audio 能力作为一条独立层接入 Nina：可测试、可替换、可扩展，并且清楚标出 Ableton API 的能力边界。

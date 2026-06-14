# v4 Agent-Ready 工作流

## 定位

这一版继续叫 v4。目标不是让 Rust 写旋律或编鼓，而是让 Rust 成为“大模型和 Ableton 之间的安全动作层”：

```text
Ableton 当前工程
-> Rust 导出上下文 JSON
-> 大模型读取上下文并生成动作 JSON
-> Rust dry-run 校验
-> Rust 通过 NinaRustBridge 执行
-> Ableton 里生成 track / clip / MIDI notes
```

这样后面封装 MCP 时，工具边界会很清楚：大模型负责音乐意图，Rust 负责协议、验证、转换和执行。

## 功能 1：Agent Context Export

命令：

```bash
cd path/to/nina_rust
cargo run -- context export --track 2 --output .nina/context_track2.json
```

它会收集：

- Live snapshot：tempo、拍号、播放状态、track 列表。
- 指定 track 摘要：1-based 用户轨道号、Remote Script 里的 0-based index、track 类型。
- device chain：轨道上的 Instrument、MIDI Effect、Audio Effect、Rack 摘要。
- drum map：如果轨道上有 Drum Rack，会生成适合大模型读取的 pad 列表。
- capabilities：当前 CLI 能执行哪些安全动作。

如果已经建立 browser index，可以顺带带上搜索结果：

```bash
cargo run -- context export \
  --track 2 \
  --browser-index .nina/sounds_index.json \
  --browser-query "cold drum rack" \
  --browser-limit 8 \
  --output .nina/context_track2.json
```

## 功能 2：完整鼓架列表

原始扫描：

```bash
cargo run -- drum scan --track 2
```

给大模型看的格式：

```bash
cargo run -- drum scan --track 2 --format agent
```

agent 格式会把 Drum Rack 展平成类似下面的结构：

```json
{
  "pad_count": 5,
  "pads": [
    {
      "pad_id": "rack0.pad0",
      "name": "Deep Kick",
      "note": 36,
      "note_name": "C1",
      "role_tags": ["kick"],
      "chains": ["Deep Kick"],
      "devices": ["Deep Kick Simpler"]
    },
    {
      "pad_id": "rack0.pad2",
      "name": "Snare Verb Wide",
      "note": 40,
      "note_name": "E1",
      "role_tags": ["snare"],
      "chains": ["Snare Verb Wide"],
      "devices": ["Snare Verb Wide Simpler"]
    }
  ],
  "role_index": {
    "kick": ["rack0.pad0"],
    "snare": ["rack0.pad1", "rack0.pad2"]
  }
}
```

这里刻意保留多个 snare、tom、perc、hat、clap、rim 等 pad。大模型应该优先引用明确的 `pad_id` 或 `note`，不要只说“用 snare”，因为同一个 Drum Rack 里可能有很多种 snare。

## 功能 3：Drum Pattern 写入

示例文件：

```bash
examples/drums/ukg_beat_pattern.json
```

执行：

```bash
cargo run -- drum write-pattern --file examples/drums/ukg_beat_pattern.json
```

JSON 格式：

```json
{
  "version": 1,
  "target": {
    "track": 2,
    "start_bar": 1,
    "end_bar": 5,
    "clip_name": "UKG Beat From Drum Map"
  },
  "events": [
    { "pad_id": "rack0.pad0", "bar": 1, "beat": 1.0, "duration": 0.25, "velocity": 112 },
    { "note": 38, "bar": 1, "beat": 2.0, "duration": 0.25, "velocity": 98 }
  ]
}
```

规则：

- `pad_id` 来自 `drum scan --format agent`。
- `note` 也可以使用，但 Rust 会检查这个 note 是否存在于当前 Drum Rack。
- `bar` / `beat` 是相对 Ableton arrangement 的小节位置。
- 也可以用 `start` 表示 clip 内相对 beat，例如 `start: 3.5`。
- Rust 会把 drum pattern 转换为现有 MIDI JSON，再调用 `write_midi_clip`。

## 功能 4：Action Plan Dry-Run 和 Apply

校验计划，不修改 Ableton：

```bash
cargo run -- plan validate --file examples/plans/v4_agent_demo_plan.json
```

执行计划：

```bash
cargo run -- plan apply --file examples/plans/v4_agent_demo_plan.json
```

计划格式：

```json
{
  "version": 1,
  "actions": [
    { "type": "create_midi_track", "name": "Agent Drums", "position": 2 },
    { "type": "create_clip", "track": 2, "start_bar": 1, "end_bar": 5, "name": "Empty Drum Clip" },
    { "type": "write_drum_pattern", "file": "../drums/ukg_beat_pattern.json" }
  ]
}
```

`validate` 会检查：

- action 不能空。
- track / position 必须从 1 开始。
- clip 的 `end_bar` 必须大于 `start_bar`。
- 引用的 MIDI JSON 或 drum pattern JSON 必须存在并且格式合法。

`apply` 会先执行同样的 validate，再逐步调用 Ableton。后续 MCP 封装时，可以把 `plan validate` 作为默认 dry-run 工具，把 `plan apply` 作为需要用户确认后才调用的工具。

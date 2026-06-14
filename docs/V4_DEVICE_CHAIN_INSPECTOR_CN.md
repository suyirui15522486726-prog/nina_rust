# v4 Device Chain Inspector

## 定位

Device Chain Inspector 是 v4 的继续增强。它的目标不是立刻修改音色参数，而是先回答一个更基础的问题：

```text
这个 track 上到底挂了什么？
```

Ableton 的 MIDI track 经常是：

```text
MIDI Effect(s) -> Instrument / Instrument Rack -> Audio Effect(s)
```

例如：

```text
Scale -> Chord -> 12 String Guitar Rack -> Saturator -> EQ Eight -> Reverb
```

如果不先扫描这个结构，后续无论是大模型控制 macro，还是 Rust CLI 修改参数，都会缺少上下文。

## 已实现命令

```bash
cargo run -- device scan --track 2
```

说明：

- `--track` 使用用户可读的 1-based 编号。
- Ableton 内部仍使用 0-based index，Rust CLI 会自动转换。
- 当前版本只读，不修改任何参数。
- 默认输出 JSON 包含 track 摘要、device 列表、参数数量和 Rack chains。
- 默认不展开每个 device 的完整参数列表，避免一个复杂音色刷出上百行参数。

如果需要查看完整参数值：

```bash
cargo run -- device scan --track 2 --include-parameters
```

## 输出结构

输出示例：

```json
{
  "track": {
    "index": 1,
    "name": "Bass",
    "has_midi_input": true,
    "has_audio_input": false
  },
  "devices": [
    {
      "index": 0,
      "name": "Scale",
      "class_name": "MidiScale",
      "role": "midi_effect",
      "is_rack": false,
      "parameter_count": 1,
      "chain_count": 0,
      "parameters": [],
      "chains": []
    }
  ],
  "device_count": 1
}
```

## 设备角色

当前 role 是保守分类：

- `midi_effect`
- `instrument`
- `instrument_rack`
- `audio_effect`
- `audio_effect_rack`
- `midi_effect_rack`

如果分类不完美，也不会丢失原始信息，因为 JSON 仍保留 `class_name` 和完整参数列表。

## Rack Chains

如果某个 device 是 Instrument Rack / Drum Rack / Effect Rack，输出中会包含：

```json
{
  "chains": [
    {
      "index": 0,
      "name": "Hammer",
      "device_count": 1,
      "devices": []
    }
  ]
}
```

当前最多递归 3 层，避免复杂 Rack 造成过大的返回结果。

## 参数展开

当使用：

```bash
cargo run -- device scan --track 2 --include-parameters
```

每个 device 会额外返回：

```json
{
  "parameters": [
    {
      "index": 0,
      "name": "Device On",
      "value": 1.0,
      "min": 0.0,
      "max": 1.0,
      "display_value": "On",
      "is_enabled": true,
      "is_quantized": true
    }
  ]
}
```

## 后续可以扩展

下一步可以加：

```bash
cargo run -- device params --track 2 --device 1
cargo run -- device macros --track 2 --device 1
cargo run -- device set-param --track 2 --device 1 --param "Mass" --value 0.72
```

但当前 v4 先保持只读扫描，这样更安全，也更适合作为课程项目中的稳定功能。

## 测试覆盖

相关测试：

```text
tests/protocol_client_contract.rs
tests/rust_cli_contract.rs
tests/remote_script/test_nina_rust_bridge.py
```

覆盖内容：

- Rust 端序列化 `device_scan_track`。
- Rust client 解码 device scan JSON。
- CLI help 中出现 `device scan`。
- Python Remote Script 能扫描 fake track 上的 MIDI effect、Instrument Rack、Audio Effect、parameters 和 rack chains。

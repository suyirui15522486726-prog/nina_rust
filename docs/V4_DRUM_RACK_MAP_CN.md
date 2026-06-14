# v4 Drum Rack Map Scanner

## 定位

Drum Rack Map Scanner 是 v4 的继续增强。它解决的问题是：

```text
这个 MIDI track 上的 Drum Rack 里，kick / snare / hihat 分别对应哪个 MIDI note？
```

这和 device 参数扫描不同。参数扫描适合调音色；编鼓需要的是 pad/note 映射。

## 命令

```bash
cargo run -- drum scan --track 2
```

说明：

- `--track` 是用户可读的 1-based 编号。
- 默认只返回有内容的 Drum Rack pads。
- 空 pad 不返回，避免一次输出 128 个空位置。
- 当前只读，不创建 clip，不写 MIDI。

如果需要查看所有空 pad：

```bash
cargo run -- drum scan --track 2 --include-empty-pads
```

## 输出示例

```json
{
  "track": {
    "index": 1,
    "name": "Drums",
    "has_midi_input": true,
    "has_audio_input": false
  },
  "rack_count": 1,
  "racks": [
    {
      "device_index": 0,
      "name": "UKG Kit",
      "class_name": "DrumGroupDevice",
      "pad_count": 128,
      "used_pad_count": 3,
      "pads": [
        {
          "index": 0,
          "name": "Kick",
          "note": 36,
          "note_name": "C1",
          "role_guess": "kick",
          "chain_count": 1
        },
        {
          "index": 1,
          "name": "Snare",
          "note": 38,
          "note_name": "D1",
          "role_guess": "snare",
          "chain_count": 1
        },
        {
          "index": 2,
          "name": "Closed Hat",
          "note": 42,
          "note_name": "F#1",
          "role_guess": "closed_hat",
          "chain_count": 1
        }
      ]
    }
  ]
}
```

## 和大模型编鼓的关系

这条链路是：

```text
Ableton Drum Rack
-> Rust drum scan
-> 得到 kick/snare/hat 的 MIDI note map
-> 大模型根据风格生成 MIDI JSON
-> Rust clip write-midi 写入 Ableton
```

例如用户说：

```text
帮我编一个 UK Garage beat
```

大模型需要先知道：

```text
kick = note 36
snare = note 38
closed_hat = note 42
```

然后才能生成正确 pitch 的 MIDI JSON。

## role_guess

当前 `role_guess` 通过 pad 名称和 chain 名称保守推断：

- `kick`
- `snare`
- `closed_hat`
- `open_hat`
- `hat`
- `crash`
- `ride`
- `tom`
- `unknown`

如果命名不标准，仍然会保留原始 `name`、`note` 和 `chains`，大模型或用户可以自行判断。

## 测试覆盖

相关测试：

```text
tests/remote_script/test_nina_rust_bridge.py
tests/protocol_client_contract.rs
tests/rust_cli_contract.rs
```

覆盖内容：

- Remote Script 扫描 fake Drum Rack pads。
- Rust DTO 序列化 `drum_scan_track`。
- Rust client 解码 drum map JSON。
- CLI help 中出现 `drum scan` 和 `--include-empty-pads`。


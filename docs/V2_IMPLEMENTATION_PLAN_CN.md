# v2.0 MIDI Pipeline 实现计划

## 阶段 1：MIDI Engine 处理能力

- 新增 `src/engine/preview.rs`
  - `MidiPreview`
  - `PitchRange`
  - `VelocityRange`
- 新增 `src/engine/transform.rs`
  - `transpose_document`
  - `quantize_document`
  - `QuantizeGrid`
- 新增测试 `tests/midi_pipeline_contract.rs`
  - preview 摘要
  - transpose 成功与越界失败
  - quantize 网格对齐

## 阶段 2：创建 MIDI 轨道

- 扩展 `src/protocol.rs`
  - `CreateMidiTrackParams`
  - `CreateTrackResult`
- 扩展 `src/client.rs`
  - `create_midi_track`
- 扩展 `remote_scripts/NinaRustBridge/__init__.py`
  - `create_midi_track`
- 扩展 Python tests
  - fake song 创建 MIDI track
  - 返回 track index / name / count

## 阶段 3：CLI 命令

- 新增 `track create-midi`
- 新增 `midi validate`
- 新增 `midi preview`
- 新增 `midi transpose`
- 新增 `midi quantize`

## 阶段 4：验证

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test -- --nocapture
python3 -m unittest tests.remote_script.test_nina_rust_bridge -v
```

## 阶段 5：文档

- 更新 README v2 命令。
- 更新版本历史。
- 统计 Rust 行数增量。

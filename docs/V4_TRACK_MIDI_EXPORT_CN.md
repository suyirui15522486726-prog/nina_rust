# v4 Track MIDI Export

## 定位

`track export-midi` 是 v4 的双向工作流补充。之前项目主要是：

```text
本地 JSON / 大模型 JSON -> Rust 校验 -> Ableton 写入 MIDI clip
```

这个功能补上反向路径：

```text
Ableton 指定 MIDI track
-> Remote Script 读取 Arrangement 里的所有 MIDI clips
-> Rust 合并为整轨时间线
-> Rust 导出为标准 .mid 文件
```

它和 `midi export` 不一样：

- `midi export`：输入是本地 Nina MIDI JSON。
- `track export-midi`：输入是 Ableton 当前工程里的第 N 轨。

## 命令

```bash
cargo run -- track export-midi --track 2 --output-dir path/to/output
```

可指定文件名：

```bash
cargo run -- track export-midi \
  --track 2 \
  --output-dir path/to/output \
  --file-name track2_full_export.mid
```

说明：

- `--track` 是用户可读的 1-based 轨道号。
- Remote Script 内部会转换为 Ableton 的 0-based track index。
- 只支持 MIDI track；audio track 会返回 track 类型错误。
- 一个轨道即使有多个 Arrangement MIDI clips，也会导出成一个 `.mid` 文件。
- note 的起点会保留在 Arrangement timeline 上的位置。例如第二个 clip 从 beat 8 开始，它里面的第一个 note 会导出为 start beat 8。

## 输出

CLI 会返回：

```json
{
  "written": true,
  "operation": "track_export_midi",
  "track": 2,
  "track_name": "Drums",
  "clip_count": 2,
  "note_count": 32,
  "output": "path/to/output/track_02_drums.mid"
}
```

## Remote Script 读取策略

Remote Script 优先尝试：

- `get_notes_extended`
- `get_notes`
- 测试环境里的 `clip.notes` fallback

这样可以兼容不同 Live 版本的 note API。Ableton Live 11 release notes 中说明 Python API / Max for Live 可以读取 MIDI clip 中的所有 notes，因此这个方向属于合理的 Live API 用法。


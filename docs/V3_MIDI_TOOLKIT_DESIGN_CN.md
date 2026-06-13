# v3.0 MIDI Toolkit 设计

## 目标

v3.0 做本地 Standard MIDI File 工具链，重点是脱离 Ableton 也能工作：

```text
本地 .mid 文件
  -> Rust 解析
  -> Nina MIDI JSON
```

以及反向：

```text
Nina MIDI JSON
  -> Rust 导出
  -> 标准 .mid 文件
```

这部分适合作为组员 A 的协作贡献：它不改变 v1/v2 的 Ableton bridge 主线，但增强了整个项目的 MIDI 工程能力。

## 用户效果

### 从本地 MIDI 转 JSON

如果用户输入：

```bash
cargo run -- midi import --input /path/to/demo.mid --track 2 --start-bar 1
```

默认输出：

```text
/path/to/demo.json
```

也就是和 `.mid` 文件同一层目录、同名 `.json`。

### 从 JSON 导出 MIDI

```bash
cargo run -- midi export --file examples/midi/phrase.json --output out/phrase.mid
```

## 范围

- 读取 SMF format 0/1。
- 支持 note on / note off。
- 提取第一个有音符的 MIDI track。
- 默认 PPQ 转换到 beat 单位。
- 导出单轨 MIDI 文件。

## 非目标

- 不分析和弦、不作曲。
- 不处理复杂 controller automation。
- 不把 MIDI 直接写入 Ableton；写入 Ableton 仍使用 v1/v2 的 `clip write-midi`。

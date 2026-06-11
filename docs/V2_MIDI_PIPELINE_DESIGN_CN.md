# v2.0 MIDI Pipeline 设计

## 目标

v2.0 不让 Rust 负责作曲生成。音乐创作来源可以是大模型、人手写 JSON、Strudel 思路、其他外部工具。Rust 负责把外部生成的 MIDI JSON 安全、稳定地送进 Ableton。

核心链路：

```text
LLM / human / external tool
  -> MIDI JSON
  -> Rust validate / preview / transform
  -> NinaRustBridge Remote Script
  -> Ableton Live MIDI clip
```

## 新增功能

### 1. 新建 MIDI 轨道

命令：

```bash
cargo run -- track create-midi --name "LLM Synth"
cargo run -- track create-midi --name "Cold Pad" --position 2
```

用途：空工程或缺少 MIDI 轨道时，CLI 可以先建立轨道，再写入外部 MIDI JSON。

### 2. MIDI JSON 校验

命令：

```bash
cargo run -- midi validate --file examples/midi/phrase.json
```

用途：在写入 Ableton 前检查 pitch、velocity、start、duration、bar range 是否合法。

### 3. MIDI JSON 预览

命令：

```bash
cargo run -- midi preview --file examples/midi/phrase.json
```

用途：不打开 Ableton 也能看到 note 数量、pitch 范围、velocity 范围、clip 长度等摘要。

### 4. MIDI JSON 转调

命令：

```bash
cargo run -- midi transpose --file phrase.json --semitones 2 --output phrase_up.json
```

用途：大模型生成后，Rust 可做非创作型变换，例如整体升高 2 个半音。

### 5. MIDI JSON 量化

命令：

```bash
cargo run -- midi quantize --file phrase.json --grid 1/16 --output phrase_q.json
```

用途：把大模型生成的不稳定时间点对齐到指定网格。

## Rust 工程点

- `struct`：MIDI preview、pitch/velocity range、transform request。
- `enum`：quantize grid、transform error、track command。
- `trait`：继续复用 `Transport` 与 `CommandPayload`。
- 泛型：继续复用 `AbletonClient<T: Transport>` 和 `CommandEnvelope<P>`。
- `Result`：所有文件 IO、JSON、MIDI transform、Remote Script 调用都通过 `Result` 传播。
- 测试：新增 engine transform tests、protocol/client tests、Remote Script fake tests。

## 非目标

- Rust 不生成旋律、不生成鼓点。
- v2 不实现 `.mid` 文件 import/export，留给后续协作成员。
- v2 不做 browser 二级索引，留给后续协作成员。

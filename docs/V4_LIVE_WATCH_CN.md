# v4.x Live Watch / Session Monitor

## 定位

这是 v4 的继续增强，不单独作为 v5。它把 Nina Rust 从“只会发控制命令的 CLI”扩展成“能观察 Ableton Live 状态变化的 CLI”。

它不生成旋律，也不替代大模型创作。它负责记录 Ableton 当前发生了什么，为未来的大模型控制层提供状态上下文。

## 已实现命令

### 1. 监控 Live 状态变化

```bash
cargo run -- live watch --interval-ms 1000 --count 10
```

说明：

- 每隔 `interval-ms` 毫秒读取一次 Ableton snapshot。
- 默认读取 10 次。
- 第一帧作为基准，不产生事件。
- 后续 snapshot 如果发生变化，会产生事件。

可记录 JSONL：

```bash
cargo run -- live watch --interval-ms 1000 --count 60 --output .nina/live_watch.jsonl
```

JSONL 的每一行都是一个 `WatchEvent`，适合后续分析或交给大模型读取。

### 2. 对比两个 snapshot 文件

```bash
cargo run -- live snapshot > .nina/snapshot_before.json
# 在 Ableton 里改 tempo、播放状态、轨道名或轨道数量
cargo run -- live snapshot > .nina/snapshot_after.json
cargo run -- live diff --before .nina/snapshot_before.json --after .nina/snapshot_after.json
```

目前支持识别：

- tempo 变化
- transport 播放状态变化
- track 新增
- track 删除
- track 重命名
- track 输入类型变化

## 分层设计

```text
src/live/
  mod.rs        对外导出 live watch 能力
  event.rs      LiveEvent 与 WatchEvent
  diff.rs       SnapshotDiff，比较两个 LiveSetSnapshot
  recorder.rs   JSONL 写入
  watcher.rs    watch loop 和轮询配置
```

`src/cli.rs` 只负责命令解析、调用 Ableton client、打印 JSON，不直接写 diff 算法。

## Rust 课程技术点

- `enum`：`LiveEvent` 表达不同 Live 状态事件。
- `struct`：`SnapshotDiff`、`WatchEvent`、`WatchOptions`、`LiveRecorder`。
- `Result`：watch、recorder、CLI 都传播错误。
- 泛型：`run_watch_loop<I, E>` 接收不同 snapshot provider，测试可以不用连接 Ableton。
- 并发/系统能力：watch loop 使用 `std::thread::sleep` 做固定间隔轮询。
- 测试：`tests/live_watch_contract.rs` 覆盖 diff、JSONL recorder、watch loop。

## 面向未来的大模型层

未来如果接入大模型，模型可以先读取：

```bash
cargo run -- live snapshot
cargo run -- live diff --before .nina/a.json --after .nina/b.json
```

然后再决定是否创建 clip、选择音色、写 MIDI。这样模型不是盲目控制 Ableton，而是可以基于状态变化做决策。


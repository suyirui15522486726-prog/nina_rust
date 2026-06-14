# v4.0 Browser Index / Sound Library Layer

## 定位

v4.0 是小组成员 B 可以负责的“音色库索引层”。它不生成 MIDI，也不让 Rust 判断音乐创意，而是把 Ableton Browser 的一级扫描结果保存成本地 JSON 索引，再提供离线搜索和随机选择能力。

## 已实现命令

### 1. 建立索引

```bash
cargo run -- browser index --root sounds --limit 200 --output .nina/sounds_index.json
```

说明：

- 需要 Ableton Live 已打开。
- 需要 `NinaRustBridge` Remote Script 已经在 Ableton 设置里选中。
- 目前只扫描根分类第一层，和 v1 的 `browser scan` 能力一致。
- 输出文件会自动创建父目录，例如 `.nina/`。

### 2. 离线搜索索引

```bash
cargo run -- browser search --index .nina/sounds_index.json --query "cold pad" --limit 10
```

说明：

- 不需要 Ableton 在线。
- 会按 name、path、tags 进行简单排名。
- 可以只搜索可加载项目：

```bash
cargo run -- browser search --index .nina/sounds_index.json --query "pad" --loadable-only
```

### 3. 离线随机选择

```bash
cargo run -- browser random --index .nina/sounds_index.json --seed 42 --loadable-only
```

说明：

- 相同索引和相同 seed 会得到相同结果，便于课堂演示和测试复现。
- 可以指定 root 过滤：

```bash
cargo run -- browser random --index .nina/sounds_index.json --root sounds --seed 42
```

## 分层设计

```text
src/browser/
  mod.rs      对外导出 browser 功能
  index.rs    索引数据结构、扫描结果转换、JSON 读写
  search.rs   查询 token 化、排名、过滤
  random.rs   seed 驱动的确定性随机选择
```

`src/cli.rs` 只负责命令解析和调用业务模块，不保存搜索算法和索引格式细节。

## 与未来大模型层的关系

未来大模型可以读取 `browser search` 或 `browser random` 的输出，再决定“选择哪个音色”。Rust v4 只负责提供稳定、可测试、可复现的数据接口。

当前链路是：

```text
Ableton Browser -> Remote Script -> Rust browser index -> local JSON -> search/random result
```

未来可以扩展为：

```text
用户自然语言 -> 大模型 -> Rust browser search/random -> Remote Script -> Ableton 加载音色
```

## 测试覆盖

对应测试文件：

```text
tests/browser_index_contract.rs
tests/rust_cli_contract.rs
```

覆盖内容：

- 从 browser scan 结果建立索引。
- 索引 JSON 保存和读取。
- 多 token 查询排名。
- `--loadable-only` 过滤。
- seed 随机选择的可复现性。
- CLI help 中出现 v4 命令。


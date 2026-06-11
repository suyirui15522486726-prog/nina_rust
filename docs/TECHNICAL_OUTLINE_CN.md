# Nina Rust 技术文档大纲

## 1. 项目定位

Nina Rust 是 Nina 项目的 Rust 重写与扩展版本。第一阶段目标不是接入大模型，也不是继续使用旧的 Max/MSP 或 macro 控制路线，而是先实现一个可独立运行的 Rust CLI 控制工具，通过本地 Ableton Remote Script 控制 Ableton Live。

本项目允许参考并改写 GitHub 上已有的 Ableton MCP 项目设计，但核心实现应转为 Rust，并围绕课程要求体现 Rust 工程实践。

## 2. 当前状态

| 模块 | 状态 | 说明 |
| --- | --- | --- |
| Cargo 项目 | 已完成 | 当前 `nina_rust` 已是基础 Rust binary 项目，包含 `Cargo.toml`、`Cargo.lock`、`src/main.rs`。 |
| Git 仓库 | 已完成 | 当前项目已有 Git 仓库，初始文件处于 staged 状态，尚无正式提交。 |
| NinaRustBridge Remote Script | 初步完成 | 已生成新的 Ableton Remote Script，并已同步到 User Library；还需要在 Ableton beta 的 Control Surface 设置中手动启用。 |
| CLI 控制功能 | 初步完成 | 已支持 `live health`、`live snapshot`、`live tempo set`、`live transport play/stop`、`clip create`、`clip write-midi`、`browser scan`。 |
| Ableton TCP Client | 初步完成 | 已实现 Rust 到 `NinaRustBridge` 的 TCP/JSON 通信、响应解析和 remote error 转换。 |
| MIDI Engine / Validation | 初步完成 | 已实现 bar 区间、MIDI JSON 文档、note 范围和 clip 长度校验；尚未实现自动旋律生成器。 |
| MCP / LLM 接入 | 暂不做 | 第一阶段不接入大模型。后续可基于同一套 Rust client/engine 再扩展 MCP server。 |

## 3. 第一阶段目标

第一阶段只做 CLI 控制，目标是让用户可以在终端直接控制 Ableton Live：

```bash
nina-rs live health
nina-rs live snapshot
nina-rs tempo set 174
nina-rs transport play
nina-rs transport stop
nina-rs clip create --track 1 --start-bar 1 --end-bar 5 --name "Nina clip"
nina-rs clip write-midi --file examples/midi/strudel_inspired_phrase.json
nina-rs browser scan --root sounds --limit 25
```

第一阶段完成后，项目应具备：

- 本地 Ableton 连接检查。
- Session / track / clip 基础读取。
- tempo、transport、track、clip、MIDI notes 基础写入。
- Rust engine 生成可写入 Ableton 的 MIDI note pattern。
- 单元测试和关键功能测试。
- `cargo fmt` 与 `cargo clippy` 可通过。

## 4. GitHub MCP 参考改写范围

### 4.1 bschoepke/ableton-live-mcp

参考点：

- `live_get`、`live_set`、`live_call`、`live_batch` 的通用 Live Object Model 思路。
- JSON-RPC 风格 request/response 结构。
- timeout、main-thread scheduling、set signature 等安全机制。

不采用：

- Agent M4L、Max/MSP、audio tap、网页 UI 相关能力。
- 直接执行 Live Python 的高风险自由代码能力作为第一阶段默认功能。

### 4.2 uisato/ableton-mcp-extended

参考点：

- 工具层分类：transport、track、clip、arrangement、cue point、device parameter、browser。
- 简单 JSON 命令协议：`type + params`。
- Remote Script 中把修改 Live 状态的命令调度到主线程执行。

改写方向：

- Python MCP server 外层改为 Rust CLI。
- 命令参数改成 Rust typed structs/enums。
- 错误处理改为 `Result<T, NinaError>`。

### 4.3 xiaolaa2/ableton-copilot-mcp

参考点：

- TypeScript 项目的工具注册、history、snapshot、rollback 设计。
- 对 clip notes、track、song、browser 等对象的分层封装。

第一阶段只参考结构，不引入 TypeScript 或 `ableton-js`。

### 4.4 ahujasid/ableton-mcp

参考点：

- 入门级 Ableton TCP Remote Script 命令集合。
- 简单易懂的 session、track、clip 控制流程。

注意点：

- 该项目 Remote Script 使用 `0.0.0.0` 监听不适合作为默认方案。本项目应只绑定 `127.0.0.1`，避免局域网暴露控制端口。

## 5. 推荐项目分层

当前项目可以从单 crate 逐步升级为 workspace。推荐结构如下：

```text
nina_rust/
  Cargo.toml
  Cargo.lock
  docs/
    TECHNICAL_OUTLINE_CN.md
    DEVELOPMENT_STATUS_CN.md
  crates/
    nina_cli/
      src/
        main.rs
        commands/
        output/
    nina_protocol/
      src/
        command.rs
        response.rs
        dto.rs
        error.rs
    nina_client/
      src/
        transport.rs
        tcp_client.rs
        ableton_client.rs
        retry.rs
    nina_engine/
      src/
        pattern/
        notes/
        planning/
        validation/
    nina_test_support/
      src/
        mock_server.rs
        fixtures.rs
  tests/
    cli_smoke_test.rs
    ableton_protocol_test.rs
```

分层类比 Java 后端：

| Java 后端概念 | Rust 项目对应层 | 职责 |
| --- | --- | --- |
| controller | `nina_cli::commands` | 解析用户命令，调用 use case，不直接拼 TCP JSON。 |
| service / use case | `nina_engine::planning`、`nina_client::ableton_client` | 编排业务流程，比如创建 track 后创建 clip 并写入 notes。 |
| dto | `nina_protocol::dto` | 定义和 Remote Script 通信的数据结构。 |
| repository / adapter | `nina_client::transport` | 负责 TCP、timeout、重试、编码解码。 |
| domain | `nina_engine::notes`、`nina_engine::pattern` | 表达 MIDI note、pattern、clip plan 等核心概念。 |
| test support | `nina_test_support` | mock Ableton server、测试 fixtures。 |

## 6. 核心模块设计大纲

### 6.1 `nina_protocol`

职责：

- 定义 Rust 与 Ableton Remote Script 之间的 JSON 协议。
- 把命令、响应、错误统一建模。

计划类型：

- `AbletonCommand`
- `CommandPayload`
- `AbletonResponse<T>`
- `RemoteStatus`
- `TrackSummary`
- `ClipSummary`
- `MidiNote`
- `NinaError`

### 6.2 `nina_client`

职责：

- 管理本地 TCP 连接。
- 发送命令并读取 JSON 响应。
- 处理 timeout、connection refused、invalid JSON、remote error。

计划接口：

- `Transport` trait。
- `TcpTransport` 实现。
- `AbletonClient<T: Transport>` 泛型 client。

### 6.3 `nina_engine`

职责：

- 不依赖具体 TCP 实现，只处理音乐控制逻辑。
- 生成 MIDI notes、clip 写入计划、基础风格 pattern。

计划功能：

- drum pattern 生成。
- bassline / chord stub。
- velocity humanize。
- beat grid quantize。
- dry-run plan 输出。
- 写入前验证：track/slot/index/tempo/range。

### 6.4 `nina_cli`

职责：

- 提供终端命令入口。
- 把用户输入转成 engine/client 调用。
- 输出人类可读文本或 JSON。

计划命令：

- `live health`
- `live snapshot`
- `tempo set`
- `transport play`
- `transport stop`
- `track list`
- `track create-midi`
- `clip create`
- `clip write-pattern`
- `clip read-notes`

## 7. Rust 作业技术点映射

### 7.1 ownership / borrowing

计划体现：

- `nina_engine` 中 pattern generator 接收不可变引用读取配置。
- client 发送命令时借用 request 数据，避免无意义 clone。
- command builder 可以消费 self 生成不可变 command，体现 ownership 转移。

### 7.2 struct / enum

计划体现：

- `struct MidiNote`、`TrackSummary`、`ClipPlan`、`TcpConfig`。
- `enum AbletonCommand`、`TransportState`、`NinaError`、`PatternKind`。

### 7.3 trait

计划体现：

- `Transport`：抽象 TCP/mock transport。
- `PatternGenerator`：抽象不同音乐 pattern 生成器。
- `RenderOutput`：抽象 text/json 输出。

### 7.4 泛型

计划体现：

- `AbletonClient<T: Transport>` 允许真实 TCP 和测试 mock 共用一套 client。
- `AbletonResponse<T>` 用泛型承载不同命令的响应数据。

### 7.5 生命周期

如需要，计划在以下位置体现：

- `CommandView<'a>` 或 formatter 借用 command/result 数据生成输出。
- 避免为了展示生命周期而硬写复杂代码；如果所有权模型自然足够清晰，生命周期只在必要处使用。

### 7.6 并发或异步

推荐采用 `tokio`：

- `tokio::net::TcpStream` 处理 TCP。
- `tokio::time::timeout` 处理超时。
- `tokio::sync::mpsc` 可用于后续批量命令或事件流。

备选方案：

- 使用标准库 `std::net::TcpStream` + thread/channel。实现更简单，但异步工程展示较弱。

## 8. 错误处理规范

项目中应尽量避免业务代码中大量 `unwrap` / `expect`。

计划错误类型：

```text
NinaError
  - ConfigError
  - ConnectionError
  - Timeout
  - JsonEncode
  - JsonDecode
  - RemoteError
  - ValidationError
```

所有关键函数返回：

```text
Result<T, NinaError>
```

CLI 层负责把错误转换成用户可读提示。

## 9. 测试计划

### 9.1 单元测试

计划覆盖：

- MIDI note 参数校验。
- pattern generator 输出数量、节拍位置、velocity 范围。
- index 转换：用户 1-based 输入到 Live 0-based index。
- JSON command 序列化与反序列化。
- 错误类型转换。

### 9.2 关键功能测试

计划覆盖：

- 使用 mock TCP server 测试 `live health`。
- 使用 mock TCP server 测试 `tempo set`。
- 使用 mock TCP server 测试 `clip write-pattern` 发送的 JSON payload。
- 测试 remote error 能正确变成 `NinaError::RemoteError`。
- 测试 timeout 能正确失败而不是卡住。

### 9.3 提交前检查

提交前必须运行：

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

如果升级为 workspace，则运行：

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

## 10. 功能完成标记规则

每个功能都按以下状态标记：

| 状态 | 含义 |
| --- | --- |
| 已完成 | 已实现、已测试、可从 CLI 使用。 |
| 初步完成 | 能跑通主路径，但测试或错误处理不完整。 |
| 开发中 | 正在实现，接口可能变化。 |
| 未开始 | 只有设计，没有代码。 |
| 暂不做 | 不属于当前阶段。 |

## 11. 第一阶段功能清单

| 功能 | 状态 | 验收标准 |
| --- | --- | --- |
| Cargo 基础项目 | 已完成 | IDEA 中可打开，包含 `Cargo.toml` 和 `src/main.rs`。 |
| workspace 分层 | 未开始 | 根 `Cargo.toml` 声明 workspace，至少拆出 CLI/protocol/client/engine。 |
| Rust module 分层 | 初步完成 | 当前已在单 crate 内拆出 `cli`、`client`、`protocol`、`engine`，后续可升级为 workspace。 |
| CLI 参数解析 | 初步完成 | 支持 `nina_rust live health`、`clip create`、`clip write-midi`、`browser scan` 等子命令。 |
| TCP transport | 初步完成 | 可连接 `127.0.0.1:9878` 并发送换行分隔 JSON。 |
| Remote response 解析 | 初步完成 | success/error 响应都能类型化处理。 |
| Ableton health check | 已完成 | CLI 能显示 Remote Script 是否在线，已用真实 Ableton 验证。 |
| Live set snapshot | 已完成 | CLI 能读取 tempo、track count、track summary，已用真实 Ableton 验证。 |
| tempo set | 初步完成 | CLI 能设置 BPM 并显示结果，已用 `120 BPM` 做低风险验证。 |
| transport play/stop | 初步完成 | CLI 能发送播放/停止命令；Remote Script 已修正返回目标状态，运行中的 Ableton 需重启或重新选择 Control Surface 才加载最新脚本。 |
| create MIDI track | 未开始 | CLI 能创建 MIDI track。 |
| arrangement MIDI clip 创建 | 已完成 | CLI 可用 `clip create --track 1 --start-bar 1 --end-bar 5` 请求创建 arrangement MIDI clip；Remote Script 会拒绝 audio track。 |
| MIDI JSON 写入 | 已完成 | CLI 可读取 `examples/midi/strudel_inspired_phrase.json`，校验 notes 后通过 Remote Script 写入 arrangement MIDI clip。 |
| browser 根级扫描 | 已完成 | CLI 可用 `browser scan --root sounds --limit 25` 扫描 Ableton browser 根分类第一层。 |
| 自动 pattern 生成 | 未开始 | 后续由 Rust engine 或大模型层生成 notes；当前只执行显式 MIDI JSON。 |
| mock / contract tests | 已完成 | 已包含 Rust mock transport/client tests、MIDI engine tests、Remote Script fake Live tests。 |
| cargo fmt/clippy/test | 已完成 | 当前已通过 `cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test`。 |

## 12. 暂不进入第一阶段的内容

- 不接入大模型。
- 不实现 MCP server。
- 不继续 Max/MSP 控制链路。
- 不把旧 `nina_core` 的 macro 设计作为主路线。
- 不默认开放远程网络端口。

## 13. 待确认问题

1. 第一阶段网络层是否确认使用 `tokio`？我建议使用 `tokio`，更符合异步工程展示，也利于未来 MCP server。
2. CLI 命令名使用英文为主是否可以？例如 `live health`、`clip write-pattern`。这样和 Rust crate/API 命名更一致。

## 14. 已确认决策

| 决策 | 结果 |
| --- | --- |
| Ableton Remote Script 路线 | 维护新的 `NinaRustBridge`，不复用旧 Nina 项目的 `NinaAgent`。 |
| Ableton 版本目标 | 优先面向用户当前计划使用的 Ableton Live beta。 |
| beta 偏好与日志目录 | `~/Library/Preferences/Ableton/Live 12.4.5b3`。 |
| Remote Script 安装路径 | 优先使用 `~/Music/Ableton/User Library/Remote Scripts/NinaRustBridge`。 |
| Ableton 设置 | 在 `Settings/Preferences -> Link, Tempo & MIDI` 中选择 `NinaRustBridge`，Input / Output 设置为 `None`。 |

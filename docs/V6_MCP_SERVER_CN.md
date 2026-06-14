# Nina Rust V6 MCP Server

V6 的目标是把 Nina Rust CLI 包装成一个 TypeScript MCP Server。后续大模型、Claude Desktop、Cursor、ChatGPT Apps 或 LangGraph agent 不需要直接拼命令行，而是可以通过 MCP tools 调用 Nina 已经验证过的能力。

## 架构定位

```text
MCP Client / Agent
  调用 Nina MCP tools

mcp-server/ TypeScript
  定义 tool schema
  校验参数
  调用 Rust CLI
  返回 JSON 文本结果

Rust CLI
  校验 MIDI / audio / media 协议
  与 Ableton Remote Script 通信
  输出 JSON

Ableton Remote Script
  调用 Ableton Live Object Model
```

这版不接 LangGraph，也不接真实大模型 API。V6 只做稳定 MCP 工具层，把 Rust CLI 暴露成结构化工具；agent loop、human-in-the-loop、tool router、LangSmith tracing 都放到后续版本继续做。

## 为什么使用 `@modelcontextprotocol/sdk` v1

V6 使用稳定的 MCP TypeScript SDK：

```json
"@modelcontextprotocol/sdk": "1.29.0"
```

入口使用 stdio transport：

```ts
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
```

这样 MCP client 可以把 Nina 当作一个本地 stdio server 启动，不需要额外开 HTTP 服务。

## 已封装工具

V6 当前提供 29 个 Rust-backed MCP tools。它们不是重新实现音乐逻辑，而是把 Rust CLI 中已有、可测试、边界清楚的能力封装成模型可调用的工具。

### Live 控制

```text
nina_live_health
nina_live_snapshot
nina_set_tempo
nina_transport_play
nina_transport_stop
```

### Track / Clip

```text
nina_create_midi_track
nina_create_audio_track
nina_create_midi_clip
nina_write_midi_clip
```

### MIDI Toolkit

```text
nina_midi_validate
nina_midi_preview
nina_midi_transpose
nina_midi_quantize
nina_midi_import
nina_midi_export
```

### Browser / Device / Drum

```text
nina_browser_scan
nina_browser_index
nina_browser_search
nina_browser_random
nina_device_scan
nina_drum_scan
```

### Audio

```text
nina_audio_import
nina_audio_effects
nina_audio_clips
nina_audio_context
nina_audio_analyze_file
nina_audio_to_midi
```

### Media Provider

```text
nina_media_plan
nina_media_status
```

V6 暂时没有把 `live watch` / `live diff` 包成 MCP tool。它们更适合后续 agent 状态机或监控流程，不适合作为第一批模型直接调用的细粒度工具。

## 本地安装

进入 MCP server 目录：

```bash
cd /absolute/path/to/nina_rust/mcp-server
npm install
npm run build
npm test
```

开发模式运行：

```bash
npm run dev
```

给 MCP client 使用时，建议先 build，然后用：

```bash
node /absolute/path/to/nina_rust/mcp-server/dist/index.js
```

## MCP Client 配置示例

不同 MCP client 的配置文件位置不同，但 stdio 配置形状类似：

```json
{
  "mcpServers": {
    "nina-rust": {
      "command": "node",
      "args": [
        "/absolute/path/to/nina_rust/mcp-server/dist/index.js"
      ],
      "env": {
        "NINA_RUST_ROOT": "/absolute/path/to/nina_rust"
      }
    }
  }
}
```

如果将项目移动到别的路径，需要同步修改：

- `args` 中的 `dist/index.js` 路径。
- `NINA_RUST_ROOT` 指向 Rust 项目根目录。

## 调用链说明

例如调用 `nina_audio_context`：

```json
{
  "track": 3
}
```

TypeScript 会映射成：

```bash
cargo run --quiet -- audio context --track 3
```

Rust CLI 负责连接 Ableton Remote Script，并把返回结果作为 JSON 输出。MCP server 再把 JSON pretty-print 成 MCP text content 返回给模型。

再比如调用 `nina_browser_search`：

```json
{
  "index": "/absolute/path/to/browser_index.json",
  "query": "cold pad",
  "limit": 5,
  "loadableOnly": true
}
```

TypeScript 会映射成：

```bash
cargo run --quiet -- browser search --index /absolute/path/to/browser_index.json --query "cold pad" --limit 5 --loadable-only
```

## 文件结构

```text
mcp-server/
  package.json
  tsconfig.json
  vitest.config.ts
  src/
    index.ts
    rustRunner.ts
    server.ts
    tools/
      definitions.ts
  test/
    rustRunner.test.ts
    serverFactory.test.ts
    toolDefinitions.test.ts
```

职责：

- `rustRunner.ts`：构造 `cargo run --quiet -- ...`，运行 Rust CLI，解析 JSON。
- `tools/definitions.ts`：定义 MCP tool 名称、Zod 参数 schema、Rust CLI 参数映射。
- `server.ts`：把 Rust-backed tools 注册到 `McpServer`。
- `index.ts`：stdio transport 入口。
- `test/`：覆盖 CLI 参数映射、JSON 解析、server factory。

## 验证命令

TypeScript 层：

```bash
cd /absolute/path/to/nina_rust/mcp-server
npm run typecheck
npm run build
npm test
npm audit --omit=dev
```

Rust / Remote Script 层：

```bash
cd /absolute/path/to/nina_rust
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test -- --nocapture
python3 -m unittest tests.remote_script.test_nina_rust_bridge -v
```

## 当前边界

- V6 不直接生成音乐。
- V6 不保存任何大模型 API key。
- V6 不替代 Rust CLI。
- V6 不直接操作 Ableton；它通过 Rust CLI 间接操作。
- V6 暂不做 LangGraph 状态机，避免 agent 层和工具层混在一起。

这个边界是刻意保留的：Rust 是稳定执行层，TypeScript 是 MCP 接入层，后续 LangGraph 才是规划、确认和 human-in-the-loop 层。

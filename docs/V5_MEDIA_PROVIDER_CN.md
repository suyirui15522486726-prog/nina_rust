# Nina Rust V5 Media Provider Layer

这一层的目标不是让 Rust 直接生成音乐，也不是在仓库里绑定某个具体大模型。它负责定义一套稳定的媒体任务协议：外部模型或外部工具负责生成音频、分轨或素材；Rust 负责校验输入、生成任务 manifest、验证输出文件是否真实存在，并把结果变成后续 Ableton / MCP / agent 都能读取的 JSON。

## 为什么需要这一层

Nina 后续会逐渐接入 TS/JS 侧的 MCP 和 LangGraph agent，但 Rust 仍然适合作为底层执行层：

- 路径、文件、manifest、音频素材这些本地资源由 Rust 严格校验。
- 大模型或音频模型只需要遵守 JSON 协议，不需要知道 Ableton Remote Script 的细节。
- TS/JS agent 可以把 Rust CLI 当作稳定 action 调用，而不是重复实现底层文件逻辑。
- 公开仓库不会包含密钥、私有模型或不可公开的逆向能力。

## 当前已实现：Dry Run Stem Provider

当前版本先实现 `dry-run` provider。它不会真的调用分轨模型，而是根据输入音频和目标 stems 生成一个 manifest，用来约定外部工具应该产出哪些文件。

命令：

```bash
cargo run -- media plan \
  --lane stem-split \
  --provider dry-run \
  --input /absolute/path/song.wav \
  --output-dir /absolute/path/stems \
  --stems vocals,drums,bass,other \
  --description "prepare stems for agent arrangement"
```

输出文件：

```text
/absolute/path/stems/nina_media_manifest.json
```

示例 manifest 形状：

```json
{
  "version": 1,
  "job_id": "stem-split-dry-run-song",
  "lane": "stem_split",
  "provider": "dry_run",
  "status": "planned",
  "input": "/absolute/path/song.wav",
  "output_dir": "/absolute/path/stems",
  "description": "prepare stems for agent arrangement",
  "manifest_path": "/absolute/path/stems/nina_media_manifest.json",
  "outputs": [
    {
      "index": 0,
      "stem_kind": "vocals",
      "lane": "audio_file",
      "path": "/absolute/path/stems/song_vocals.wav"
    }
  ]
}
```

检查外部工具是否已经把文件放到位：

```bash
cargo run -- media status \
  --manifest /absolute/path/stems/nina_media_manifest.json
```

如果目标 stem 文件还不存在，会返回：

```json
{
  "status": "missing_outputs",
  "expected_count": 4,
  "existing_count": 0
}
```

如果外部工具已经生成了全部文件，会返回 `ready`。

## 架构边界

```text
src/media/
  MediaProvider trait
  MediaRequest / MediaJobManifest / MediaVerification
  DryRunMediaProvider
  stem list parsing
  manifest save/load

src/cli.rs
  media plan
  media status

tests/media_provider_contract.rs
  Rust provider 合约测试

tests/media_cli_contract.rs
  CLI manifest 读写和状态检查测试
```

`MediaProvider` 是一个 trait，所以后续可以添加真实 provider，而不需要改 CLI 外层协议：

```text
DryRunMediaProvider
DemucsCliProvider
SunoDownloadProvider
LocalModelProvider
```

## 和未来 TS/JS 的关系

推荐后续分层：

```text
TS/JS MCP Server
  接收大模型 tool call
  管理 human-in-the-loop 问题
  调用 nina_rust CLI

Rust CLI
  校验文件和协议
  控制 Ableton
  生成/验证 manifest

Ableton Remote Script
  执行 Live Object Model 能力
```

这样 TS/JS 负责 agent 编排，Rust 负责可测试的底层动作。Rust 不会拖后腿，反而能让 agent 的 action 更稳定。

## 后续可扩展方向

- `demucs-cli` provider：调用开源分轨 CLI，生成 vocals/drums/bass/other。
- `suno-audio` provider：接收外部下载好的 wav/mp3，做本地校验和 Ableton 导入。
- `midi-json` lane：对 GPT 生成的 Nina MIDI JSON 做统一 manifest 管理。
- `audio-file` lane：统一管理外部模型生成的单个音频文件。

当前版本先把协议和测试搭稳，不在公开仓库里放任何模型密钥或不可公开能力。

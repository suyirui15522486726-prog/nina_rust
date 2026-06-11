# Ableton Remote Script 复现配置指南

本文档面向想从 GitHub 下载 Nina Rust 项目并在自己电脑上复现 Ableton 连接的人。

目标是让 Ableton Live 加载本项目提供的 `NinaRustBridge` Remote Script，并让后续 Rust CLI 可以通过本机 TCP 端口控制 Ableton。

## 1. 架构说明

本项目第一阶段不是 Max/MSP，也不是 Ableton Extensions SDK，而是：

```text
Rust CLI / Rust Engine
        |
        | TCP JSON, localhost only
        v
Ableton Remote Script: NinaRustBridge
        |
        v
Ableton Live Object Model
```

其中：

- Rust 代码负责 CLI、协议、测试、音乐生成和后续 MCP 扩展。
- `NinaRustBridge` 是 Ableton 侧必须加载的 Python Remote Script。
- Remote Script 只监听 `127.0.0.1:9878`，不开放局域网端口。
- 第一阶段不需要 npm；npm 只用于可选的安装脚本管理或未来 TypeScript/MCP/Extensions SDK 路线。

## 2. 需要下载或安装什么

### 必需

1. Ableton Live

推荐使用 Live 12 beta 或 Live 12 正式版。  
如果你使用 Live 12.4.5 public beta，也可以继续使用本文档流程。

2. Git

用于从 GitHub 克隆本项目。

```bash
git --version
```

3. Rust 工具链

用于运行后续 Rust CLI。

```bash
rustc --version
cargo --version
```

如果没有 Rust，安装：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

4. Nina Rust 项目源码

项目发布到 GitHub 后，复现者使用：

```bash
git clone https://github.com/<your-github-name>/nina_rust.git
cd nina_rust
```

如果不使用 Git，也可以在 GitHub 页面点击：

```text
Code -> Download ZIP
```

解压后进入项目目录。

### 可选

1. Node.js / npm

本项目当前不要求 npm。  
只有在以下情况才需要 npm：

- 你想用 npm script 管理 Remote Script 的复制安装。
- 你未来要接入 TypeScript MCP server。
- 你要研究 `ableton-js` 或 Ableton Extensions SDK。

检查 npm：

```bash
node --version
npm --version
```

2. Ableton MCP 参考仓库

普通使用者不需要下载这些仓库。  
开发者如果想理解本项目设计来源，可以参考：

- `bschoepke/ableton-live-mcp`
- `uisato/ableton-mcp-extended`
- `ahujasid/ableton-mcp`
- `xiaolaa2/ableton-copilot-mcp`

这些仓库只作为设计参考，不是运行 Nina Rust 的依赖。

## 3. 项目内 Remote Script 位置

克隆项目后，Remote Script 应位于：

```text
nina_rust/
  remote_scripts/
    NinaRustBridge/
      __init__.py
      README.md
```

Ableton 下拉框里显示的名字来自文件夹名，所以文件夹必须叫：

```text
NinaRustBridge
```

不要只复制 `__init__.py` 文件到 `Remote Scripts` 根目录。必须保留文件夹结构：

```text
Remote Scripts/
  NinaRustBridge/
    __init__.py
```

## 4. Ableton User Library 路径

Ableton 官方推荐第三方 Remote Script 放进 User Library 下的 `Remote Scripts` 文件夹。

### macOS

默认路径：

```text
~/Music/Ableton/User Library/Remote Scripts
```

本机示例：

```text
~/Music/Ableton/User Library/Remote Scripts
```

### Windows

默认路径通常是：

```text
C:\Users\<username>\Documents\Ableton\User Library\Remote Scripts
```

如果你的 User Library 改过位置，以 Ableton 里显示的 User Library 实际路径为准。

## 5. 手动安装 Remote Script

### macOS / Linux shell

在项目根目录执行：

```bash
mkdir -p "$HOME/Music/Ableton/User Library/Remote Scripts"
rsync -a --delete \
  remote_scripts/NinaRustBridge/ \
  "$HOME/Music/Ableton/User Library/Remote Scripts/NinaRustBridge/"
```

检查安装结果：

```bash
find "$HOME/Music/Ableton/User Library/Remote Scripts/NinaRustBridge" -maxdepth 2 -type f -print
```

应该看到：

```text
.../NinaRustBridge/__init__.py
.../NinaRustBridge/README.md
```

### Windows PowerShell

在项目根目录执行：

```powershell
$target = "$env:USERPROFILE\Documents\Ableton\User Library\Remote Scripts\NinaRustBridge"
New-Item -ItemType Directory -Force -Path $target
Copy-Item -Recurse -Force ".\remote_scripts\NinaRustBridge\*" $target
```

检查：

```powershell
Get-ChildItem "$env:USERPROFILE\Documents\Ableton\User Library\Remote Scripts\NinaRustBridge"
```

## 6. 可选：用 npm 管理复制命令

本项目第一阶段不依赖 npm。  
如果维护者希望让复现者用 npm 一键安装 Remote Script，可以在项目根目录额外提供 `package.json`：

```json
{
  "name": "nina-rust-tools",
  "private": true,
  "scripts": {
    "install:ableton-script:mac": "mkdir -p \"$HOME/Music/Ableton/User Library/Remote Scripts\" && rsync -a --delete remote_scripts/NinaRustBridge/ \"$HOME/Music/Ableton/User Library/Remote Scripts/NinaRustBridge/\"",
    "check:ableton-script:mac": "find \"$HOME/Music/Ableton/User Library/Remote Scripts/NinaRustBridge\" -maxdepth 2 -type f -print"
  }
}
```

复现者可以执行：

```bash
npm run install:ableton-script:mac
npm run check:ableton-script:mac
```

注意：

- 这只是把复制命令包装成 npm script。
- 它不是 TypeScript MCP server。
- 它不替代 Rust 的 `cargo build` / `cargo test`。
- Windows 路径包含空格和反斜杠，建议单独写 PowerShell 脚本，不要硬塞进同一个 npm command。

## 7. 在 Ableton 中启用 Control Surface

安装 Remote Script 后，必须重启 Ableton Live。

然后在 Ableton 中操作：

1. 打开 `Settings` 或 `Preferences`。
2. 进入 `Link, Tempo & MIDI`。
3. 找到 `Control Surface` 区域。
4. 在一个空槽位中选择：

```text
NinaRustBridge
```

5. 同一行 `Input` 选择：

```text
None
```

6. 同一行 `Output` 选择：

```text
None
```

为什么是 `None`：

- `NinaRustBridge` 不是实体 MIDI 控制器。
- 它通过本地 TCP socket 与 Rust CLI 通信。
- 第一阶段不需要选择 MIDI 键盘、声卡或虚拟 MIDI 端口。

## 8. 验证连接

当 Ableton 成功加载 `NinaRustBridge` 后，它会监听：

```text
127.0.0.1:9878
```

在 Rust CLI 完成前，可以用 Python 临时验证：

```bash
python3 - <<'PY'
import json
import socket

payload = {"type": "health_check", "params": {"from": "manual-test"}}

with socket.create_connection(("127.0.0.1", 9878), timeout=3) as sock:
    sock.sendall((json.dumps(payload) + "\n").encode("utf-8"))
    print(sock.recv(4096).decode("utf-8"))
PY
```

预期返回类似：

```json
{"status": "success", "result": {"ok": true, "name": "NinaRustBridge", "host": "127.0.0.1", "port": 9878, "echo": {"from": "manual-test"}}}
```

## 9. 日志与排错

### 下拉框里看不到 NinaRustBridge

检查：

- 是否重启过 Ableton。
- 文件夹是否叫 `NinaRustBridge`。
- 是否存在 `NinaRustBridge/__init__.py`。
- 是否放在 User Library 的 `Remote Scripts` 下。
- 是否把 `remote_scripts` 整个项目目录错误复制进去了。

正确：

```text
Remote Scripts/NinaRustBridge/__init__.py
```

错误：

```text
Remote Scripts/remote_scripts/NinaRustBridge/__init__.py
Remote Scripts/__init__.py
```

### Ableton beta 日志

macOS 上 Live beta 日志通常在：

```text
~/Library/Preferences/Ableton/Live 12.4.5b3/Log.txt
```

可以查看最近错误：

```bash
tail -n 120 "$HOME/Library/Preferences/Ableton/Live 12.4.5b3/Log.txt"
```

如果不是 beta 版，把目录中的版本号替换成你实际使用的版本。

### 端口连不上

检查 Ableton 是否正在监听：

```bash
lsof -nP -iTCP:9878 -sTCP:LISTEN
```

如果没有输出，通常说明：

- Ableton 没有启动。
- `NinaRustBridge` 没有被选为 Control Surface。
- Remote Script 启动时报错。
- 端口被其他程序占用。

### 端口被占用

查看占用者：

```bash
lsof -nP -iTCP:9878
```

如果确实冲突，需要修改 `remote_scripts/NinaRustBridge/__init__.py` 中的：

```python
PORT = 9878
```

并同步脚本、重启 Ableton。

## 10. 与 npm / ableton-js / Extensions SDK 的关系

不要混淆三条路线：

| 路线 | 是否本项目第一阶段必需 | 说明 |
| --- | --- | --- |
| NinaRustBridge Remote Script | 必需 | 本项目当前采用的 Ableton 连接层。 |
| Rust Cargo | 必需 | 后续 CLI、client、engine 都由 Rust/Cargo 管理。 |
| npm scripts | 可选 | 可以用来包装复制 Remote Script 的命令。 |
| `ableton-js` | 不必需 | Node/TypeScript 控制 Ableton 的另一条路线。 |
| MCP TypeScript SDK | 不必需 | 未来如果做 TS MCP server 才需要。 |
| Ableton Extensions SDK | 不必需 | Live 12.4.5 public beta 的 JS/TS 扩展路线，和 Remote Script 路线不同。 |

如果只是复现 Nina Rust 第一阶段，请按本文档安装 `NinaRustBridge`，再使用 Rust/Cargo 构建 CLI。

## 11. 参考链接

- Ableton 官方：Installing third-party remote scripts  
  https://help.ableton.com/hc/en-us/articles/209072009-Installing-third-party-remote-scripts
- Ableton 官方：Using Control Surfaces  
  https://help.ableton.com/hc/en-us/articles/209774285-Using-Control-Surfaces
- Ableton Extensions SDK public beta  
  https://ableton.github.io/extensions-sdk/
- npm：`@modelcontextprotocol/sdk`  
  https://www.npmjs.com/package/@modelcontextprotocol/sdk
- GitHub：`ableton-js`  
  https://github.com/leolabs/ableton-js

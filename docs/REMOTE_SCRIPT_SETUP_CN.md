# NinaRustBridge Remote Script 配置流程

## 1. 结论

本项目维护新的 Ableton Remote Script，名字暂定为：

```text
NinaRustBridge
```

第一阶段不复用旧 Nina 项目的 `NinaAgent`，也不走 Max/MSP 或 macro 控制路线。Rust CLI 后续会连接这个新的 Remote Script。

你打算使用 Ableton Live beta 版本开发。当前机器上已发现 beta 偏好目录：

```text
~/Library/Preferences/Ableton/Live 12.4.5b3
```

后续排查日志、Control Surface 加载错误时，优先查看这个 beta 目录下的日志。

## 2. 参考来源

本流程参考了：

- Ableton 官方第三方 Remote Script 安装说明。
- `uisato/ableton-mcp-extended` 的 Remote Script 安装和 Ableton 设置流程。
- `ahujasid/ableton-mcp` 的 Control Surface 启用流程。
- `xiaolaa2/ableton-copilot-mcp` / AbletonJS 的 User Library `Remote Scripts` 路线。

关键共识：

- Live 11 以后 Remote Script 必须使用 Python 3。
- 推荐把第三方 Remote Script 放进 Ableton User Library 的 `Remote Scripts` 文件夹。
- 启用时需要在 Ableton 的 `Settings/Preferences -> Link, Tempo & MIDI` 中选择 Control Surface。
- 对于这种本地 socket bridge，Input 和 Output 通常设置为 `None`。

## 3. 你的电脑上的 Ableton 情况

已检测到 Ableton app：

```text
/Applications/Ableton Live 12 Suite.app
```

app bundle 当前显示版本：

```text
12.4.1 (2026-05-20_fbe5fe99c9)
```

已检测到 beta 偏好目录：

```text
~/Library/Preferences/Ableton/Live 12.4.5b3
```

beta 日志位置：

```text
~/Library/Preferences/Ableton/Live 12.4.5b3/Log.txt
```

已检测到 User Library：

```text
~/Music/Ableton/User Library
```

推荐安装目标目录：

```text
~/Music/Ableton/User Library/Remote Scripts/NinaRustBridge
```

注意：你的电脑上也存在旧式路径：

```text
~/Library/Preferences/Ableton/Live 12.3.8/User Remote Scripts
~/Library/Preferences/Ableton/Live 12.4.1/User Remote Scripts
~/Library/Preferences/Ableton/Live 12.4.5b3/User Remote Scripts
```

但本项目优先使用 User Library 路径，因为它不绑定具体 Live 小版本，也更适合 beta / stable 并存的情况。

## 4. 未来项目内文件夹安排

后续代码建议放在：

```text
nina_rust/
  remote_scripts/
    NinaRustBridge/
      __init__.py
      README.md
  crates/
    nina_cli/
    nina_protocol/
    nina_client/
    nina_engine/
```

安装到 Ableton 时，只复制：

```text
remote_scripts/NinaRustBridge/
```

到：

```text
~/Music/Ableton/User Library/Remote Scripts/NinaRustBridge/
```

## 5. 你需要在 Ableton 里手动操作的步骤

### Step 1：确认 Remote Scripts 文件夹存在

在 Finder 里按：

```text
Cmd + Shift + G
```

粘贴这个路径：

```text
~/Music/Ableton/User Library
```

如果里面没有 `Remote Scripts` 文件夹，就新建一个：

```text
Remote Scripts
```

最终路径应为：

```text
~/Music/Ableton/User Library/Remote Scripts
```

### Step 2：确认项目内 `NinaRustBridge`

项目内 Remote Script 已生成：

```text
<repo-root>/remote_scripts/NinaRustBridge/__init__.py
```

它是 Ableton 侧的薄 Python bridge，后续 Rust CLI 会通过 `127.0.0.1:9878` 连接它。

### Step 3：复制或同步到 Ableton User Library

当前已经同步到 User Library。目标结构应保持为：

```text
~/Music/Ableton/User Library/Remote Scripts/
  NinaRustBridge/
    __init__.py
    README.md
```

注意：下拉框显示的名字来自文件夹名，所以文件夹必须叫 `NinaRustBridge`。

### Step 4：重启 Ableton Live beta

安装或更新 Remote Script 后，建议重启 Ableton Live beta。

如果 Live 已经打开，最可靠方式是：

1. 退出 Ableton Live beta。
2. 重新打开你要用于开发的 Ableton Live beta。

### Step 5：在 Ableton 里启用 Control Surface

打开 Ableton Live beta 后：

1. 进入 `Settings` / `Preferences`。
2. 打开 `Link, Tempo & MIDI`。
3. 找到 `Control Surface` 区域。
4. 在一个空槽位里选择：

```text
NinaRustBridge
```

5. 同一行的 `Input` 设置为：

```text
None
```

6. 同一行的 `Output` 设置为：

```text
None
```

### Step 6：确认脚本加载成功

脚本加载成功后，后续版本会在 Ableton 状态栏显示类似：

```text
NinaRustBridge listening on 127.0.0.1:9878
```

或在 beta 日志中出现对应日志：

```text
~/Library/Preferences/Ableton/Live 12.4.5b3/Log.txt
```

## 6. 以后 Rust CLI 的连接方式

Rust CLI 默认连接：

```text
host = 127.0.0.1
port = 9878
```

初始验证命令计划为：

```bash
nina-rs live health
```

预期返回：

```json
{
  "status": "success",
  "result": {
    "ok": true,
    "name": "NinaRustBridge"
  }
}
```

## 7. 常见问题

### Ableton 下拉框里看不到 NinaRustBridge

检查：

- 文件夹是否叫 `NinaRustBridge`。
- 文件是否是 `NinaRustBridge/__init__.py`。
- 是否放在 `~/Music/Ableton/User Library/Remote Scripts/` 下面。
- 是否重启过 Ableton Live beta。
- `__init__.py` 是否有 Python 语法错误。

### 选择后没有监听端口

检查：

- Control Surface 是否已经选中 `NinaRustBridge`。
- Input / Output 是否为 `None`。
- 端口是否被占用。
- beta 日志里是否有 Python exception。

### 是否需要 MIDI 设备输入输出

第一阶段不需要外部 MIDI 控制器，所以 Input / Output 设置为 `None`。

这个 Remote Script 是本地 TCP bridge，不是物理 MIDI 键盘映射。

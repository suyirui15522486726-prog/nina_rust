# B 同学 v4 提交流程说明

## 你拿到的是什么

这份压缩包是 `nina_rust` 的 v4 功能包。它建立在 v3 之后，主要包含：

- v4 browser index / search / random
- live watch / diff
- device chain inspector
- Drum Rack pad map scanner
- agent-ready context export
- action plan validate / apply
- drum pattern JSON 写入
- track export-midi 整轨 MIDI 导出

这份包不包含 `.git`、`target`、`.idea`、Python 缓存等本地文件。

## 最重要的提交顺序

B 同学可以先解压、阅读、测试，但不要在 A 同学的 v3 PR 合并前提交 v4 PR。

正确顺序：

```text
1. A 同学先提交并合并 v3
2. B 同学从合并 v3 后的最新 main 新建 v4 分支
3. B 同学把这份 v4 包里的代码应用进去
4. B 同学运行测试
5. B 同学提交 v4 PR
```

原因：v4 是建立在 v3 的 MIDI toolkit 之上的。如果 B 先提交，GitHub 会把 A 的 v3 改动也显示在 B 的 PR 里，协作记录会乱。

## Windows 上推荐流程

假设 GitHub 仓库已经存在，并且 A 的 v3 已经合并到 `main`。

### 1. 克隆仓库

```bat
cd %USERPROFILE%\Desktop
git clone <你的仓库地址> nina_rust
cd nina_rust
```

### 2. 确认 main 是最新

```bat
git switch main
git pull origin main
```

### 3. 新建 v4 分支

```bat
git switch -c feature/v4-agent-ableton-workflow
```

### 4. 应用 v4 包

把压缩包里的文件复制到当前 `nina_rust` 仓库目录，覆盖同名文件。

注意：只复制项目文件，不要复制额外的系统缓存文件。如果压缩包是干净的，直接全选复制即可。

### 5. 检查改动

```bat
git status
```

应看到 `src`、`tests`、`docs`、`examples`、`remote_scripts` 等文件变化。

### 6. 运行测试

```bat
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

如果本机没有 Ableton 或 Python 环境，可以先跳过 Remote Script 真实连接测试，但至少要跑 Rust 测试。

如果 Python 可用，可以继续跑：

```bat
python -m unittest tests.remote_script.test_nina_rust_bridge -v
```

### 7. 提交 v4

```bat
git add .
git commit -m "feat: add v4 agent-ready Ableton workflow"
git push -u origin feature/v4-agent-ableton-workflow
```

然后在 GitHub 上创建 PR，目标分支选 `main`。

## B 同学 PR 里可以怎么写

标题：

```text
feat: add v4 agent-ready Ableton workflow
```

说明：

```markdown
## Summary
- Add v4 Ableton browser/device/drum inspection workflow.
- Add agent context export and action plan validation/apply flow.
- Add Drum Rack pattern writing and whole-track MIDI export.

## Test
- cargo fmt --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test
- python -m unittest tests.remote_script.test_nina_rust_bridge -v
```

## 约束条件

B 同学除了等 A 的 v3 合并后再提交，还需要注意：

- 必须从“包含 v3 的最新 main”新建 v4 分支。
- 不要把 v5 的 MCP / LangGraph / Ableton Extensions SDK 内容混进这个 PR。
- 不要提交 `target/`、`.idea/`、`.git/`、`__pycache__/`、`.DS_Store`。
- 不要提交本地路径、密钥、token、API key。
- PR 里要强调 v4 是在 v3 MIDI toolkit 基础上的扩展。

## 本地 Ableton Remote Script 提醒

如果要实际连接 Ableton，需要把 Remote Script 放到 Ableton User Library 的 Remote Scripts 目录，并在 Ableton 的 Link, Tempo & MIDI 设置中选择 `NinaRustBridge`。

如果 CLI 报：

```text
Unknown NinaRustBridge command
```

通常表示 Ableton 还加载着旧脚本。重启 Ableton 后再测。


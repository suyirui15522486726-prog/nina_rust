# Nina Rust V5 GitHub 上传指南

本文档用于 V5 本地包上传 GitHub 前的操作说明。当前协作顺序建议保持为：

```text
A 同学：提交并合并 V3 PR
B 同学：基于 V3 提交并合并 V4 PR
你：基于已经合并 V4 的 main 提交 V5 PR
```

如果 B 同学的 V4 PR 还没有提交或合并，V5 先不要提交 PR 到 GitHub。你可以先保存 zip 包，等 V4 合并后再按下面流程操作。

## 一、确认本地环境

在终端检查：

```bash
git --version
gh --version
cargo --version
rustc --version
```

如果 `gh` 没登录：

```bash
gh auth login
gh auth status
```

## 二、V4 合并前不要做的事

暂时不要执行：

```bash
git push
gh pr create
```

原因：V5 是基于 V4 的后续版本。如果 V4 还没进入 main，V5 先提交会让 GitHub 历史看起来像跳过了 B 同学的工作，也容易产生冲突。

## 三、V4 合并后提交 V5 的推荐流程

进入你的 GitHub 仓库本地目录：

```bash
cd /path/to/your/nina_rust_repo
```

拉取最新 main：

```bash
git checkout main
git pull origin main
```

创建 V5 分支：

```bash
git checkout -b feature/v5-audio-layer
```

把 V5 zip 解压后的项目内容复制到当前仓库。注意不要复制这些目录或文件：

```text
.git/
target/
.idea/
*.iml
__pycache__/
.DS_Store
```

复制完成后检查改动：

```bash
git status
```

## 四、提交前验证

在项目根目录运行：

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test -- --nocapture
python3 -m unittest tests.remote_script.test_nina_rust_bridge -v
```

如果 Windows 环境没有 `python3`，可以尝试：

```cmd
py -m unittest tests.remote_script.test_nina_rust_bridge -v
```

## 五、提交 V5

```bash
git add .
git status
git commit -m "feat: add v5 audio and media provider layer"
git push -u origin feature/v5-audio-layer
```

创建 PR：

```bash
gh pr create \
  --base main \
  --head feature/v5-audio-layer \
  --title "feat: add v5 audio and media provider layer" \
  --body "V5 adds audio track/clip operations, audio context scanning, experimental audio-to-MIDI support, and a media provider manifest layer for future MCP/agent integration."
```

## 六、PR 描述建议

可以写：

```text
This PR adds the V5 audio layer for Nina Rust.

Included:
- create audio track
- import local audio file into Ableton arrangement
- scan audio track effect chain
- scan audio clips and export audio context JSON
- experimental Ableton audio-to-MIDI bridge
- media provider manifest layer for future MCP / agent integration
- Rust contract tests and Remote Script fake Live tests

Verification:
- cargo fmt --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test -- --nocapture
- python3 -m unittest tests.remote_script.test_nina_rust_bridge -v
```

## 七、课程协作说明建议

如果老师看 GitHub 记录，可以这样解释版本分工：

```text
V3: MIDI Toolkit, handled by teammate A.
V4: Browser/device/drum/live context improvements, handled by teammate B.
V5: Audio layer and media provider architecture, handled by project initiator.
```

这样能体现协作顺序，同时 V5 也自然延续 V4 的 Ableton context 能力。

# 公开提交检查清单

本文件用于发布到 GitHub 前的自查。

## 不应提交的内容

- `.idea/`
- `*.iml`
- `target/`
- `__pycache__/`
- `.DS_Store`
- 本机绝对路径，例如带真实 macOS 用户名的路径
- API key、token、password、private key

## 当前公开路径写法

文档中应使用：

```text
~/Music/Ableton/User Library/Remote Scripts/NinaRustBridge
~/Library/Preferences/Ableton/<Live Version>/Log.txt
<repo-root>/remote_scripts/NinaRustBridge
```

不要使用具体用户名路径。

## 推荐扫描方式

```bash
rg -n --hidden --glob '!target/**' --glob '!.git/**' '<local-username>' .
git status --short --ignored
```

还应使用 GitHub secret scanning 或本地 secret scanner 检查常见 API key / token / private key 格式。

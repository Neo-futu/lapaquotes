# scripts

本地开发辅助脚本目录。生产部署相关脚本见 [`ops/`](../ops/)。

| 脚本 | 说明 |
|------|------|
| [`ingest/`](./ingest/) | Outlook `fixed income` 存款报价邮件 → `DepositQuote` |
| [`bi/`](./bi/) | 飞书多维表格日更（`daily_sync.py`） |
| [`db/`](./db/) | 创建 `lapa` 库、应用表结构（`init-db.ps1` / `init-db.sh`） |
| `gen-db-client.sh` | 根据 `schema.sql` + `queries/` 生成 Cornucopia 客户端 |
| `build-slate.sh` | 构建 Admin 富文本编辑器（Slate.js / rspack） |
| `dev-site.sh` / `dev-site.ps1` | 以 watch 模式启动前台站点 |
| `dev-admin.sh` / `dev-admin.ps1` | 以 watch 模式启动管理后台 |

Windows（含 conda `(base)` 终端）若提示找不到 `cargo`，先执行：

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
```

或直接用上面的 `.ps1` 脚本（会自动补 PATH）。

在仓库根目录执行，例如：

```sh
./scripts/gen-db-client.sh
./scripts/build-slate.sh
./scripts/dev-site.sh
```

Windows 可用 Git Bash / WSL，或对照脚本内容手动执行等价命令。

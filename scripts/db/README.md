# 数据库脚本

为本机 PostgreSQL 创建 `lapa` 库并写入项目表结构。

## 文件

| 文件 | 说明 |
|------|------|
| `01_create_database.sql` | 创建数据库 `lapa`（若不存在） |
| `02_schema.sql` | 引入根目录 [`schema.sql`](../../schema.sql) 建表 |
| `03_drop_tables.sql` | **危险**：删除全部业务表（保留库） |
| `init-db.ps1` | Windows 一键初始化（推荐） |
| `init-db.sh` | Bash / Git Bash / WSL 一键初始化 |

## 表一览

`User`、`Session`、`Settings`、`Content`、`ContentImage`、`Product`、`ProductImage`、`PostCategory`、`Post`、`Ssr`、`Holiday`

香港公众假日种子（Quotes 页日历）：

```powershell
& $psql -U postgres -h 127.0.0.1 -d lapa -f scripts\db\04_seed_holidays_hk.sql
```

## 快速开始（Windows）

```powershell
# 在仓库根目录执行；按提示输入 postgres 密码，或先设环境变量：
$env:PGPASSWORD = "你的密码"
.\scripts\db\init-db.ps1
```

可选参数：

```powershell
.\scripts\db\init-db.ps1 -PgHost 127.0.0.1 -Port 5432 -User postgres -Database lapa
```

成功后会在仓库根目录写入/更新 `.env` 中的 `PG__URL`（若尚无该行）。

## 手动执行

```powershell
$psql = "D:\Program Files\PostgreSQL\18\bin\psql.exe"
$env:PGPASSWORD = "你的密码"

& $psql -U postgres -h 127.0.0.1 -p 5432 -d postgres -f scripts\db\01_create_database.sql
& $psql -U postgres -h 127.0.0.1 -p 5432 -d lapa -f schema.sql
```

## 初始化业务数据

建表完成后：

```sh
RUST_LOG=debug cargo cli settings-init
RUST_LOG=debug cargo cli user-add admin yourpassword
```

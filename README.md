# LapaQuotes

OTC 报价平台：多机构 / 多期限 / 多币种的**存款与债券**日报价终端，配套 Admin、AI Agent，以及飞书多维表格 BI 同步。

基于 Rust（Leptos + Axum）与 PostgreSQL（Cornucopia SQL-first）。仓库：[`Neo-futu/lapaquotes`](https://github.com/Neo-futu/lapaquotes)。

## 能力概览

| 模块 | 说明 |
|------|------|
| **前台 site** | `/` 报价终端（Deposits / Bonds）、日期与机构筛选、最优 Bid/Ask 高亮 |
| **Admin** | 机构、存款/债券报价 CRUDL；文章/产品/设置（模板能力仍保留） |
| **Agent** | Rig 多模型对话；报价查询 tools；飞书 / WhatsApp 渠道 |
| **BI → 飞书** | 平台库 → `BiQuoteFact` → OTC报价台账 / 推送表 / 仪表盘视图 |

多币种（USD / CNY / HKD / EUR）、资产类别（存款 / 债券）为一级维度。

## 文档

| 文档 | 内容 |
|------|------|
| [doc/bi.md](doc/bi.md) | **飞书 BI 架构与每日同步**（必读） |
| [doc/agent.md](doc/agent.md) | Agent API、渠道、环境变量 |
| [doc/ingestion.md](doc/ingestion.md) | 邮件入库与幂等 upsert 约定 |
| [doc/architecture.md](doc/architecture.md) | Workspace 与数据流 |
| [doc/features.md](doc/features.md) | 功能清单 |
| [doc/tech-stack.md](doc/tech-stack.md) | 技术栈 |

## 仓库结构

```text
site (根包)     前台 :4141
admin/          后台 :4143
agent/          AI + BI（可挂 site 或独立 :4151）
cli/            settings-init / user-add
cornucopia/     生成的 DB 客户端
queries/        手写 SQL
schema.sql      表结构（含 DepositQuote / BondQuote / Bi*）
scripts/bi/     飞书日更 Python
doc/            设计与运维文档
commit.py       一键 add / commit / push（Neo-futu）
```

## 快速开始

### 1. 环境

- Rust（见 `rust-toolchain.toml`）、PostgreSQL
- 复制环境变量：`.env.example` → `.env`（**勿提交** `.env`）

```env
PG__URL=postgresql://postgres:postgres@127.0.0.1:5432/lapa
```

### 2. 数据库

```sh
# Windows
./scripts/db/init-db.ps1
# 或
psql -h 127.0.0.1 -U postgres -d lapa -f schema.sql
```

多币种迁移（若库已有旧唯一键）：

```sh
psql ... -f scripts/db/05_multi_currency.sql
```

### 3. 初始化

```sh
cargo run -p cli -- settings-init
cargo run -p cli -- user-add
```

### 4. 本地开发

```sh
python scripts/dev.py            # site :4141 + admin :4143（watch）
python scripts/dev.py --only site
```

或：

```sh
cargo leptos watch -p site
cargo leptos watch -p admin
```

富文本编辑器（Admin 需要时）：

```sh
cd slate && npm i && npm run build && cd ..
```

## 飞书 BI 同步

完整说明见 **[doc/bi.md](doc/bi.md)**。

`.env` 仅需：

```env
FEISHU_BITABLE_ACCESS_TOKEN=pt-...
FEISHU_BITABLE_APP_TOKEN=...
FEISHU_BITABLE_API_BASE=https://base-api.feishu.cn
```

```sh
# 应用 BI 表
python scripts/bi/apply_schema.py

# 每日同步（最新报价日，或指定日期）
python scripts/bi/daily_sync.py
python scripts/bi/daily_sync.py 2026-10-08

# 或 CLI
cargo run -p agent --example bi_sync --features standalone -- daily
cargo run -p agent --example bi_sync --features standalone -- sync 2026-10-08
cargo run -p agent --example bi_sync --features standalone -- views 2026-10-08
```

## Git（Neo-futu）

本仓 local 身份为 Neo-futu；`origin` 使用 SSH Host `github-neo`（勿与 zhairen 的 `github.com` 混淆）。

```sh
python commit.py "你的 commit message"
```

## 上游说明

前端/Admin 骨架源自 [alexichepura/lapa](https://github.com/alexichepura/lapa)（Leptos + Axum starter）。本仓库在此之上发展为 **OTC 报价 + Agent + 飞书 BI** 产品线，以本文档与 `doc/` 为准。

## License

[MIT](LICENSE-MIT)

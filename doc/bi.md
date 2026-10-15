# Lapa BI：平台报价 → 飞书多维表格

本文说明 **数据库表 → BI 层 → 飞书 Base（数据表 + 仪表盘）** 的架构、业务逻辑，以及 **每日如何更新**。

相关 Base：[OTC价格台账](https://balancev.feishu.cn/base/O563bA6LkakSfmsTU0ScIWNFn3g)

| 资源 | 说明 | 链接 |
|------|------|------|
| OTC报价台账 | 明细表（逐条存款/债券） | `table=tblxiDoHI6BN7PFK` |
| 最新报价推送 | 摘要表（按机构聚合文案） | `table=tblV8hKHncQCkpOB` |
| 原商品仪表盘 | 模板遗留（铜铝塑钢密封胶） | [`blkRgeGoMaQuEZaB`](https://balancev.feishu.cn/base/O563bA6LkakSfmsTU0ScIWNFn3g?table=blkRgeGoMaQuEZaB) |
| OTC报价仪表盘 | 由模板复制的 OTC 壳 | [`blk2g1IiEYJs0U2f`](https://balancev.feishu.cn/base/O563bA6LkakSfmsTU0ScIWNFn3g?table=blk2g1IiEYJs0U2f) |

---

## 1. 架构总览

```text
┌─────────────────────────────────────────────────────────────┐
│  平台层（已有）                                               │
│  Institution / DepositQuote / BondQuote                       │
│  Web /quotes · Admin · Agent Chat（只读这些表）               │
└───────────────────────────┬─────────────────────────────────┘
                            │ extract（按报价日）
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  BI 层  agent/src/bi                                         │
│  ┌────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ BiQuoteFact    │  │ BiBitable*      │  │ BiSyncJob    │ │
│  │ 事实/暂存/镜像  │  │ 表·视图·字段·   │  │ 任务审计     │ │
│  │                │  │ 仪表盘目录       │  │              │ │
│  └───────┬────────┘  └────────▲────────┘  └──────────────┘ │
│          │                    │ discovery                   │
│          │ push               │                             │
│          ▼                    │                             │
│  bitable/ 适配器（仅 ACCESS_TOKEN + APP_TOKEN + API_BASE）   │
└───────────────────────────┬─────────────────────────────────┘
                            │ HTTPS base-api.feishu.cn
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  飞书多维表格 Base                                           │
│  OTC报价台账 · 最新报价推送 · OTC报价仪表盘（图表）            │
└─────────────────────────────────────────────────────────────┘
```

### 分层职责

| 层 | 目录 / 表 | 职责 |
|----|-----------|------|
| 平台 | `DepositQuote` / `BondQuote` | OTC 业务真相源；邮件入库脚本写入（见 `doc/ingestion.md`） |
| BI Store | `BiQuoteFact` 等 | 按日抽取事实、记录 Feishu `record_id`、任务日志、目录缓存 |
| BiService | `agent/src/bi/service.rs` | 编排：catalog → migrate → sync → dashboard |
| Bitable 适配器 | `agent/src/bi/bitable/` | 发现表/视图/仪表盘、改表结构、upsert 记录 |
| 日更脚本 | `scripts/bi/daily_sync.py` | Cron：调 HTTP，失败则 `cargo … bi_sync` |

**原则**：不要在 `.env` 写死 `TABLE_ID` / `VIEW_ID`。只需：

```env
FEISHU_BITABLE_ACCESS_TOKEN=pt-...   # 插件 → 自定义插件 → 获取授权码
FEISHU_BITABLE_APP_TOKEN=O563bA6LkakSfmsTU0ScIWNFn3g
FEISHU_BITABLE_API_BASE=https://base-api.feishu.cn
```

`pt-` **必须**走 [`base-api.feishu.cn`](https://base-api.feishu.cn)；`open.feishu.cn` 会报 `99991668`。

---

## 2. 数据流与业务逻辑

### 2.1 同步键（幂等主键）

| 类型 | sync_key | 示例 |
|------|----------|------|
| 存款 | `D\|日期\|机构\|期限\|币种` | `D\|2026-10-07\|Acme Bank\|6M\|USD` |
| 债券 | `B\|日期\|机构\|代码\|币种` | `B\|2026-10-07\|Globex\|US1234567890\|USD` |
| 推送摘要 | `G\|日期\|机构\|kind` | `G\|2026-10-07\|Acme Bank\|deposit` |

### 2.2 一日同步（`BiService::sync_day`）

1. **解析日期**：入参或平台库最新报价日（`recent_quote_dates`）。
2. **Extract**：`DepositQuote` + `BondQuote` → `Vec<QuoteFact>`。
3. **Load BI**：`BiQuoteFact` upsert（按 `sync_key`）。
4. **Push 台账**：按日 `contains |YYYY-MM-DD|` 查远程 → create / update / 删多余行；回写 `bitable_record_id`。
5. **Push 推送表**：按机构+类型写摘要行 + 全日汇总行。
6. **记 Job**：`BiSyncJob`（created/updated/deleted/facts）。

### 2.3 台账列（`quote_ledger`）

| 列 | 类型 | 来源 |
|----|------|------|
| 同步键 | 文本（主键） | sync_key |
| 日期 | 日期 | quote_date |
| 机构 / 类型 / 期限 / 币种 | 文本/单选 | 平台字段；类型=`存款`/`债券` |
| Bid / Ask | 数字 | 存款利率；债券优先收益率否则价格 |
| 代码 / 名称 / 备注 | 文本 | 债券为主；备注含方向/票息/到期 |

### 2.4 推送表（`quote_digest`）

供消息/看板摘要：`机构`、`类型`、`报价日`、`条数`、`摘要`。

### 2.5 仪表盘（图表）

飞书开放能力（[`列出仪表盘`](https://open.feishu.cn/document/server-docs/docs/bitable-v1/app-dashboard/list?lang=zh-CN) / 复制仪表盘）：

| 能力 | personalBaseToken (`pt-`) |
|------|---------------------------|
| 列出仪表盘 | ✅ |
| 复制仪表盘 | ✅ |
| 创建/修改单个图表 Block | ❌（`base/v3/.../blocks` 对 `pt-` 不可用） |

因此代码侧：

1. `ensure_dashboard`：若不存在「OTC报价仪表盘」，则从模板 [`blkRgeGoMaQuEZaB`](https://balancev.feishu.cn/base/O563bA6LkakSfmsTU0ScIWNFn3g?table=blkRgeGoMaQuEZaB) **复制**壳子。
2. 为台账准备**带相等筛选的视图**（仪表盘里「等于」条件靠视图实现，见 §2.6）。
3. **图表数据源需在飞书 UI 重绑**到对应视图（见 §4）。每日 `sync` 更新数据，并刷新「最新日*」视图的日期筛选。

目录落库：`BiBitableDashboard`。

### 2.6 台账筛选视图（仪表盘相等条件）

`pt-` **不能**用 API 改仪表盘图表的筛选/维度；可行做法是：在「OTC报价台账」上建网格视图（`is` 相等筛选），图表数据源选该视图。

| 视图名 | 筛选（AND） | 用途 |
|--------|-------------|------|
| 全部报价 | （无） | 全量 |
| 存款报价 / 债券报价 | 类型 = 存款/债券 | 按资产类别 |
| 1M USD 存款跨机构 | 类型=存款 · 期限=1M · 币种=USD | 各机构 1M Bid 比较 |
| Acme 美元存款各期限 | 类型=存款 · 机构=Acme Bank · 币种=USD | 单机构曲线 |
| USD 存款 / CNY 存款 | 类型=存款 · 币种 | 分币种 |
| 最新日存款 / 最新日债券 / 最新日 USD 存款 / 最新日 Acme 美元存款 / 最新日 1M USD 存款 | 上列 + 日期=同步日 | 当日切片（每次 sync 刷新）；雷达图用「最新日 USD 存款」 |

API 注意：单选字段的 `value` 必须传 **option id** 的 JSON 字符串（如 `"[\"optXxx\"]"`），不能传显示名。

```sh
cargo run -p agent --example bi_sync --features standalone -- views 2026-10-08
```

---

## 3. 每日如何更新多维表格

### 3.1 前置（一次性）

```sh
# 1) BI 库表
python scripts/bi/apply_schema.py

# 2) 商品模板 → OTC 表结构 + 清理旧视图
cargo run -p agent --example bi_sync --features standalone -- migrate

# 3) 仪表盘壳 + 图表视图
cargo run -p agent --example bi_sync --features standalone -- dashboard

# 4) 全链路校验（抽库 → 写飞书 → 读回）
cargo run -p agent --example bi_sync --features standalone -- verify
```

在飞书中按 §4 把仪表盘图表绑到 OTC 字段（只需一次）。

### 3.2 每日例行（推荐 Cron）

平台侧邮件入库完成后执行：

```sh
# 默认同步「库中最新报价日」
python scripts/bi/daily_sync.py

# 或指定日
python scripts/bi/daily_sync.py 2026-10-07
# 等价：BI_SYNC_DATE=2026-10-07 python scripts/bi/daily_sync.py
```

脚本逻辑：

1. 优先 `POST {AGENT_BI_URL}/catalog/refresh` → 必要时 `/migrate` → `/sync`
2. Agent 未启动时 fallback：`cargo run -p agent --example bi_sync --features standalone -- daily`

可选环境变量：

| 变量 | 默认 |
|------|------|
| `AGENT_BI_URL` | `http://127.0.0.1:4141/agent/bi` |
| `BI_SYNC_DATE` | 空 = 最新报价日 |

### 3.3 HTTP 一览（Agent 已启动时）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/agent/bi/catalog` | BI 中缓存的表/字段 |
| POST | `/agent/bi/catalog/refresh` | 从飞书重拉目录 |
| POST | `/agent/bi/migrate` | 改表结构（慎用，会清旧列数据） |
| POST | `/agent/bi/sync` | `{"date":"YYYY-MM-DD"}` 或 `{}` |
| POST | `/agent/bi/verify` | sync + 读回校验 |
| GET | `/agent/bi/facts?date=` | BI 事实 |
| GET | `/agent/bi/jobs` | 最近任务 |
| GET | `/agent/bi/dashboards` | 仪表盘目录 |
| POST | `/agent/bi/dashboards/ensure` | 确保 OTC 仪表盘存在 |

### 3.4 Windows 任务计划示例

```text
程序: python
参数: D:\vsproject\lapaquotes\scripts\bi\daily_sync.py
起始于: D:\vsproject\lapaquotes
触发器: 每个工作日 18:30（邮件入库之后）
```

---

## 4. 仪表盘图表：一次性 UI 重绑（必做）

开放 API 无法用 `pt-` 改图表组件，请在网页完成。仪表盘入口：[OTC报价仪表盘](https://balancev.feishu.cn/base/O563bA6LkakSfmsTU0ScIWNFn3g?table=blk2g1IiEYJs0U2f)。

### 4.1 USD 多期限 × 多机构 Bid 雷达图（推荐）

目标：同一报价日、仅 USD 存款；**雷达轴 = 期限**，**每条多边形 = 一家机构**，数值 = **Bid**。

1. 打开 [OTC报价仪表盘](https://balancev.feishu.cn/base/O563bA6LkakSfmsTU0ScIWNFn3g?table=blk2g1IiEYJs0U2f) → 右上角编辑 / 添加图表。  
2. 图表类型选 **雷达图**。  
3. **数据源**：表「OTC报价台账」→ 视图 **「最新日 USD 存款」**  
   （筛选已固定：类型=存款 · 币种=USD · 日期=最近同步日；勿选「全部报价」。）  
4. 字段映射（飞书文案可能叫「分类 / 系列 / 数值」或「维度 / 对比 / 指标」）：

| 角色 | 字段 | 说明 |
|------|------|------|
| 分类 / 雷达轴（角） | **期限** | O/N、1W、1M、3M、6M… |
| 系列 / 对比（多条线） | **机构** | Acme、Globex… 各一条多边形 |
| 数值 / 指标 | **Bid** | 聚合选 **平均值** 或 **最大值**（同日同机构同期限应只有 1 条，两者等价） |

5. 不要再在图表里加「类型 / 币种」筛选（视图已滤好）。若轴顺序乱，到台账视图里按 `期限` 手动排序，或接受飞书默认顺序。  
6. 保存。之后每次 `sync` / `daily_sync` 会刷新「最新日 USD 存款」的日期，雷达图跟当日 Bid。

**变体**：只要某一家机构的期限雷达 → 数据源改「最新日 Acme 美元存款」，去掉「系列=机构」，只留 期限 + Bid。

### 4.2 其他常用图

| 图表意图 | 数据源视图 | 维度 | 指标 |
|----------|------------|------|------|
| 报价条数（存款 vs 债券） | 最新日存款（或全表+类型） | 类型 | 记录数 |
| 各机构 1M Bid 柱状 | 最新日 1M USD 存款 | 机构 | Bid 平均 |
| 单机构期限曲线 | 最新日 Acme 美元存款 | 期限 | Bid / Ask |
| 按日趋势 | USD 存款 | 日期 | Bid 平均 |

模板原件仍在：[铜铝…仪表盘](https://balancev.feishu.cn/base/O563bA6LkakSfmsTU0ScIWNFn3g?table=blkRgeGoMaQuEZaB)（可忽略）。

---

## 5. BI 库表一览

| 表 | 用途 |
|----|------|
| `BiBitableTable` | 飞书表目录 + role（`quote_ledger` / `quote_digest`） |
| `BiBitableView` | 视图 ID/名 |
| `BiBitableField` | 字段名/类型 |
| `BiBitableDashboard` | 仪表盘 block_id / role / url |
| `BiQuoteFact` | 日事实 + 远程 record_id |
| `BiSyncJob` | 同步任务审计 |

定义见仓库根目录 `schema.sql`。

---

## 6. 代码地图

```text
agent/src/bi/
  mod.rs / service.rs / extract.rs / http.rs / models.rs
  store/          # Postgres CRUD
  bitable/
    client.rs     # HTTP
    discovery.rs  # 表/视图/字段 + role
    migrate.rs    # 商品模板 → OTC 列 + 清视图
    push.rs       # ledger / digest upsert
    dashboard.rs  # 列表 / 复制 / 视图准备
scripts/bi/
  apply_schema.py
  daily_sync.py
  cleanup_views.py
```

---

## 7. 故障排查

| 现象 | 处理 |
|------|------|
| `99991668 Invalid access token` | 确认 `API_BASE=https://base-api.feishu.cn` 且 token 为 `pt-` |
| sync 报 missing field | 跑一次 `migrate`（会清空该表再建模） |
| 图表空白 | 数据源仍指向旧商品字段 → 按 §4 重绑 |
| HTTP 连接失败 | `daily_sync.py` 会自动 fallback 到 cargo example |
| 授权过期 | 飞书多维表格 → 插件 → 自定义插件 → 重新获取授权码写入 `.env` |

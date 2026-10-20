# OTC 存款报价邮件入库（Outlook → Postgres）

从本机 Outlook 组邮箱 **`fixed income` / Inbox** 拉取各银行每日存款 indication，**按机构独立 Python 脚本**解析后幂等写入 `lapa."DepositQuote"`。契约见 [`doc/ingestion.md`](../../doc/ingestion.md)。

## 依赖

```sh
pip install -r scripts/ingest/requirements.txt
```

需已登录桌面 Outlook（COM / `pywin32`），且能看到 `fixed income` 存储。

## 用法

```sh
# 全部已配置机构（最近周五 / 指定日）
python scripts/ingest/run_ingest.py --friday
python scripts/ingest/run_ingest.py --date 2026-10-16 --dry-run

# 只跑某一机构 key
python scripts/ingest/run_ingest.py --date 2026-10-16 --bank abchk --dry-run

# 直接跑该机构脚本（等价）
python scripts/ingest/banks/abchk.py --date 2026-10-16 --dry-run
```

环境变量：`PG__URL`（默认 `postgresql://postgres:postgres@127.0.0.1:5432/lapa`，可读根目录 `.env`）。

## 新增机构邮件

1. 在 `banks/` 新建 `<key>.py`，实现 `parse(...)`（可参考 `abchk.py` / `ccbhk.py`），底部用 `_runner.run_bank` 支持单跑。
2. 在 [`institutions.yaml`](./institutions.yaml) 增加一条：`emails` / `subject_contains` / `script: <key>`。
3. `python scripts/ingest/banks/<key>.py --date YYYY-MM-DD --dry-run` 验证后去掉 `--dry-run` 入库。

同一 SMTP 发多机构（如 CCB HK / Asia）时，用不同 `subject_contains` + **各自脚本**（`ccbhk.py` / `ccba.py`）区分。

## 已支持机构

| 机构 | 特征 | 脚本 |
|------|------|------|
| NCB | today's indicative deposit rates | `banks/ncb.py` |
| ABC HK | ABCHK 正文三币种 | `banks/abchk.py` |
| HXB HK | 中文期限表 | `banks/hxb.py` |
| CCB HK | 主题 CCBHK | `banks/ccbhk.py` |
| CCB Asia | 主题 CCBA | `banks/ccba.py` |
| UOB HK | xlsx 附件 | `banks/uob.py` |
| CITIC | Tenor 竖表 | `banks/citic.py` |

## Watch 三场景测试（ABCHK 重扫 / HXB 空表 / 以此為準）

见 [`TEST_CASES_WATCH.md`](./TEST_CASES_WATCH.md)。一键：

```sh
python scripts/ingest/test_cases_watch_reingest.py --date 2026-10-20
python scripts/ingest/reset_institution_day.py --date 2026-10-20 --bank abchk --unmark-watch
```

## 文件

| 路径 | 作用 |
|------|------|
| `run_ingest.py` | 全机构调度 |
| `registry.py` | yaml 匹配 + 动态加载 `banks.<script>` |
| `institutions.yaml` | 发件人/主题 → 机构 → script |
| `banks/*.py` | **每机构一脚本** |
| `banks/_vertical_tenor.py` | CCB/CITIC 共用版式 |
| `outlook_fetch.py` | Outlook COM |
| `db.py` / `common.py` | 入库与规范化 |

单利率 indication 写入 `bid_rate`；`CNH` → `CNY`。入库后可用 `python scripts/bi/daily_sync.py` 同步飞书。

## 实时监测（项目内注册 + 自动启动）

配置写在 **`watch_config.yaml`**（项目内，不是 Windows 服务）。注册后，`python scripts/dev.py` 会自动拉起 `watch_outlook`。

```sh
# 1) 注册到本项目（enabled + auto_with_dev）
python scripts/ingest/register_watch.py register
python scripts/ingest/register_watch.py status

# 2) 启动站点时自动带上 Outlook watch
python scripts/dev.py
python scripts/dev.py --no-outlook-watch          # 本次不启 watch

# 手动启停
python scripts/ingest/register_watch.py start
python scripts/ingest/register_watch.py stop
python scripts/ingest/register_watch.py configure --poll-seconds 45
```

按日刷新 / 常驻：

```sh
python scripts/ingest/watch_outlook.py --today
python scripts/ingest/watch_outlook.py --date 2026-10-20 --force
python scripts/ingest/watch_outlook.py                        # 读 yaml 常驻
python scripts/ingest/run_ingest.py                           # 默认今天批量入库
```

| 文件 | 作用 |
|------|------|
| `watch_config.yaml` | 项目内开关与参数 |
| `register_watch.py` | 注册/配置/启停 |
| `watch_outlook.py` | 监测主循环 |
| `pipeline.py` | 分类 → 入库 → 飞书 + day-check |
| `watch_state.py` | 已处理 `EntryID` |
| `db.py` | upsert + `IngestDayCheck` / `IngestMailLog` |

前台日历下方 **Ingest Check** 展示当日各机构是否已 OK（表 `IngestDayCheck`）。

- **本地**：`DepositQuote` 唯一键 `(institution_id, quote_date, term, currency)` → 存在则更新
- **飞书**：BI `sync_key` 日同步 → 存在则更新台账/每日最高，否则新增
- 债券主题目前只识别并 skip（尚无 bond 解析脚本）

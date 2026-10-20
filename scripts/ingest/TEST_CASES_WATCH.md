# Watch / Ingest 三场景测试

覆盖：库表删除后重扫、错误空邮件、以此為準 订正邮件。Ingest Check 按 `mail_received_at` **早→晚**排序。

## 前置

- Outlook 已登录，组邮箱 `fixed income` / `Inbox` 有当日邮件
- Postgres `PG__URL` 可用；site `:4141` 建议在跑（飞书 HTTP sync）
- watch 状态库：`scripts/ingest/.watch_state/processed.sqlite3`

```sh
# 可选：确认监测配置
python scripts/ingest/register_watch.py status
```

## 一键回归

```sh
python scripts/ingest/test_cases_watch_reingest.py --date 2026-10-20
# 跳过飞书：
python scripts/ingest/test_cases_watch_reingest.py --date 2026-10-20 --no-bi
```

结果 JSON：`scripts/ingest/.watch_state/test_cases_<date>.json`，退出码 `0` = 三 case 全过。

### 2026-10-20 实测（全过）

| Case | 结果 |
|------|------|
| 1 ABCHK | clear → re-ingest **ok** 36 rows · Check `11:35` · Feishu updated |
| 2 HXB 空表 | **error** 0 quotes · Check `11:47` · `empty quote mail (0 rows)` |
| 3 HXB 以此為準 | **ok** 38 rows · Check `11:53` · Feishu updated（覆盖 case 2） |

手工重置某一机构（删报价 + Ingest Check → pending + 忘记 EntryID）：

```sh
python scripts/ingest/reset_institution_day.py --date 2026-10-20 --bank abchk --unmark-watch
python scripts/ingest/reset_institution_day.py --date 2026-10-20 --bank hxb --unmark-watch
```

---

## Case 1 — ABCHK 删除后重扫入库

**邮件主题**：`***ABCHK - Indicative Deposit Rates - 20 Oct 2026***`

| 步骤 | 动作 | 期望 |
|------|------|------|
| 1 | `clear_institution_day(abchk)`：删当日 `DepositQuote`，`IngestDayCheck` → `pending` | 报价 0 行，Check ○ pending |
| 2 | `WatchState.unmark_institution(abchk)` | 同一 EntryID 可再被 watch 处理 |
| 3 | `process_mail` / watch catch-up 发现该邮件 | 解析 → upsert `DepositQuote` → Feishu sync |
| 4 | Ingest Check | ✓ ok，时间为 Outlook `ReceivedTime`（如 `11:35`） |

说明：watch 默认按 EntryID 幂等；**仅当库被清掉后需要重扫时**才 unmark。日常新邮件不需要。

---

## Case 2 — 华夏空表邮件（错误/无数据）

**邮件主题**（无「以此為準」）：  
`华夏银行香港分行机构客户定期存款报价（美元）及 货币汇率参考价格 - 2026年10月20日`  
（Outlook 可能为繁体「華夏銀行…」；发件人常为 `hkggordonchan@hxb.com.cn`，表体利率为空 / `TBC`）

| 步骤 | 动作 | 期望 |
|------|------|------|
| 1 | 先 `clear_institution_day(hxb)` + unmark | 模拟「今天还没正确入库」 |
| 2 | 只处理上述空表邮件 | `parse` → 0 rows → status **`error`** |
| 3 | 不写 `DepositQuote` | 报价仍 0 |
| 4 | Ingest Check | ✗ error，`last_message` 含 `empty quote mail (0 rows)`，可显示收件 `HH:MM` |

要点：空表不再记为 quietly `skip`，前台会显示未 OK。

---

## Case 3 — 「以此為準」订正邮件 upsert

**邮件主题**：  
`(以此為準) 华夏银行香港分行机构客户定期存款报价（美元）及 货币汇率参考价格 - 2026年10月20日`

| 步骤 | 动作 | 期望 |
|------|------|------|
| 1 | 在 Case 2 之后（Check 为 error）处理该邮件 | 解析出有效利率行 |
| 2 | upsert 本地 `DepositQuote` | `rows_n > 0` |
| 3 | sync 飞书 BI | `bi_sync.ok == true`（`--no-bi` 时跳过） |
| 4 | Ingest Check | ✓ ok（error → ok） |

排序：`--today` / `refresh_date` 处理邮件时，带「以此為準 / 以此为准」的主题**最后处理**，保证订正覆盖同日前序草稿。

---

## 验收清单（UI）

打开 `/quotes`，选当日：

1. Ingest Check 列表按收件时间早→晚（如 NCB 08:58 … ABC HK 11:35）
2. Case 1 后 ABC HK 为 ✓ + 时间
3. Case 2 单独跑完时 HXB 为 ✗
4. Case 3 后 HXB 为 ✓，报价表与飞书「每日最高报价」有华夏数据

## 相关代码

| 模块 | 作用 |
|------|------|
| `pipeline.py` | 0 行 → `error`；`is_authoritative_subject` / `mail_process_key` |
| `db.py` | `clear_institution_day`；`record_ingest_attempt` 状态机 |
| `watch_state.py` | `unmark_entry` / `unmark_institution` |
| `watch_outlook.py` | 日刷新按权威主题排序；error 也 mark |
| `reset_institution_day.py` | CLI 重置 |
| `test_cases_watch_reingest.py` | 三 case 自动化 |

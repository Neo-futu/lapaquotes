# OTC 报价入库契约（Python 脚本对接说明）

本文档描述报价数据库表结构，供外部 Python 脚本（解析各 OTC 机构邮件的存款/债券每日报价并入库）对接。Rust 站点（前台 `/quotes`、admin、AI agent）只读这些表。

**Outlook 存款报价实装**：[`scripts/ingest/`](../scripts/ingest/)（组邮箱 `fixed income` → **每机构独立脚本** `banks/*.py` + `institutions.yaml` → `DepositQuote`）。周五样例：

```sh
python scripts/ingest/run_ingest.py --friday
python scripts/ingest/run_ingest.py --date 2026-10-16 --dry-run
python scripts/ingest/banks/abchk.py --date 2026-10-16 --dry-run   # 单机构
```

新增机构：新建 `scripts/ingest/banks/<key>.py`（实现 `parse`），并在 `institutions.yaml` 登记发件人/主题与 `script: <key>`。详见 [`scripts/ingest/README.md`](../scripts/ingest/README.md)。

**实时监测**（项目内配置注册，非 Windows 服务；`dev.py` 启动时自动拉起）：

```sh
python scripts/ingest/register_watch.py register   # 写入 watch_config.yaml
python scripts/dev.py                              # 自动启动 watch_outlook
python scripts/ingest/register_watch.py status
```

**每日机构 Check**：入库时写入 `IngestDayCheck` / `IngestMailLog`；前台 `/quotes` 日历下方按选中日期显示各机构 ✓/○/✗（按收件时间早→晚）。迁移：`scripts/db/08_ingest_day_check.sql`。重扫/空表/以此為準 三场景见 [`scripts/ingest/TEST_CASES_WATCH.md`](../scripts/ingest/TEST_CASES_WATCH.md)。

见 `scripts/ingest/register_watch.py` / `watch_config.yaml` / `pipeline.py`。

## 1. 连接

```
postgresql://postgres:postgres@127.0.0.1:5432/lapa
```

建表以仓库根目录 `schema.sql` 为单一来源（幂等 `CREATE TABLE IF NOT EXISTS`）。

应用 schema（任选其一）：

```sh
# psql（Windows 安装在 D:\Program Files\PostgreSQL\18\bin，如未入 PATH 用全路径）
"/d/Program Files/PostgreSQL/18/bin/psql.exe" "$PG__URL" -f schema.sql
# 或 Python psycopg2
python -c "import psycopg2; c=psycopg2.connect('postgresql://postgres:postgres@127.0.0.1:5432/lapa'); cur=c.cursor(); cur.execute(open('schema.sql',encoding='utf-8').read()); c.commit()"
```

## 2. 表结构

### 2.1 Institution（机构）

| 列 | 类型 | 说明 |
|----|------|------|
| id | TEXT PK | 建议 Python 端 `uuid.uuid4().hex` |
| created_at | TIMESTAMP DEFAULT now | 无需填写 |
| name | TEXT NOT NULL UNIQUE | 机构全名（唯一，作为去重键） |
| short_name | TEXT NULL | 缩写（前台优先显示） |
| email | TEXT NULL | 报价来源邮箱（便于按邮件发件人匹配机构） |
| is_active | BOOLEAN DEFAULT true | 前台机构下拉只列 active 机构 |

### 2.2 DepositQuote（存款报价）

| 列 | 类型 | 说明 |
|----|------|------|
| id | TEXT PK | 建议 `uuid.uuid4().hex` |
| created_at | TIMESTAMP DEFAULT now | 无需填写 |
| quote_date | DATE NOT NULL | 报价日期（邮件日期/报价日） |
| institution_id | TEXT NOT NULL FK→Institution | 级联删除 |
| term | TEXT NOT NULL | 期限文本，如 `O/N`、`T/N`、`1W`、`2W`、`1M`、`3M`、`6M`、`1Y` |
| term_sort | INTEGER NOT NULL DEFAULT 0 | 排序权重，见 2.4 |
| currency | TEXT NOT NULL DEFAULT 'USD' | 币种 |
| bid_rate | NUMERIC NULL | 存款 Bid 利率（%） |
| ask_rate | NUMERIC NULL | 存款 Ask 利率（%） |
| note | TEXT NULL | 备注 |

**幂等 upsert 唯一键**：`UNIQUE (institution_id, quote_date, term, currency)`

### 2.3 BondQuote（债券报价）

| 列 | 类型 | 说明 |
|----|------|------|
| id | TEXT PK | 建议 `uuid.uuid4().hex` |
| created_at | TIMESTAMP DEFAULT now | 无需填写 |
| quote_date | DATE NOT NULL | 报价日期 |
| institution_id | TEXT NOT NULL FK→Institution | 级联删除 |
| bond_code | TEXT NOT NULL | 债券代码 / ISIN |
| bond_name | TEXT NULL | 债券名称 / 发行人 |
| currency | TEXT NOT NULL DEFAULT 'USD' | 币种 |
| maturity_date | DATE NULL | 到期日 |
| coupon_rate | NUMERIC NULL | 票面利率（%） |
| bid_yield | NUMERIC NULL | Bid 收益率 |
| ask_yield | NUMERIC NULL | Ask 收益率 |
| bid_price | NUMERIC NULL | Bid 价格 |
| ask_price | NUMERIC NULL | Ask 价格 |
| direction | TEXT NULL | 方向：`Bid` / `Ask` / `Both` |
| note | TEXT NULL | 备注 |

**幂等 upsert 唯一键**：`UNIQUE (institution_id, quote_date, bond_code)`

### 2.4 term_sort 推荐映射

| term | sort | term | sort |
|------|------|------|------|
| O/N  | 0    | 6M   | 7    |
| T/N  | 1    | 9M   | 8    |
| 1W   | 2    | 1Y   | 9    |
| 2W   | 3    | 18M  | 10   |
| 1M   | 4    | 2Y   | 11   |
| 2M   | 5    | 3Y   | 12   |
| 3M   | 6    | 5Y   | 13   |
|      |      | 10Y  | 14   |

未收录的期限自定义整数即可；数值越小越靠前（前台与 admin 列表按 `term_sort` 升序）。

## 3. 幂等 upsert 示例

同一封邮件重复导入不会产生重复行（`ON CONFLICT` 命中唯一键后更新）：

```sql
-- 机构（按 name 幂等）
INSERT INTO "Institution" (id, name, short_name, email)
VALUES (:id, :name, :short_name, :email)
ON CONFLICT ("name") DO UPDATE
SET short_name = EXCLUDED.short_name,
    email = EXCLUDED.email;

-- 存款报价
INSERT INTO "DepositQuote"
    (id, quote_date, institution_id, term, term_sort, currency, bid_rate, ask_rate, note)
VALUES
    (:id, :quote_date, :institution_id, :term, :term_sort, :currency, :bid_rate, :ask_rate, :note)
ON CONFLICT (institution_id, quote_date, term, currency) DO UPDATE
SET bid_rate  = EXCLUDED.bid_rate,
    ask_rate  = EXCLUDED.ask_rate,
    note      = EXCLUDED.note;

-- 债券报价
INSERT INTO "BondQuote"
    (id, quote_date, institution_id, bond_code, bond_name, currency, maturity_date,
     coupon_rate, bid_yield, ask_yield, bid_price, ask_price, direction, note)
VALUES
    (:id, :quote_date, :institution_id, :bond_code, :bond_name, :currency, :maturity_date,
     :coupon_rate, :bid_yield, :ask_yield, :bid_price, :ask_price, :direction, :note)
ON CONFLICT (institution_id, quote_date, bond_code) DO UPDATE
SET bond_name     = EXCLUDED.bond_name,
    currency      = EXCLUDED.currency,
    maturity_date = EXCLUDED.maturity_date,
    coupon_rate   = EXCLUDED.coupon_rate,
    bid_yield     = EXCLUDED.bid_yield,
    ask_yield     = EXCLUDED.ask_yield,
    bid_price     = EXCLUDED.bid_price,
    ask_price     = EXCLUDED.ask_price,
    direction     = EXCLUDED.direction,
    note          = EXCLUDED.note;
```

## 4. 机构匹配建议

邮件发件人 → 机构：

1. 先按 `"Institution".email = 发件人` 精确匹配；
2. 未命中则按名称模糊匹配或人工确认后建立映射（配置文件/表均可）；
3. 新机构用 3. 中的 upsert 建立，再用返回/查询到的 `id` 写报价。

## 5. Python 端参考（psycopg2）

```python
import psycopg2, uuid

conn = psycopg2.connect("postgresql://postgres:postgres@127.0.0.1:5432/lapa")
cur = conn.cursor()

def institution_id_by_email(email, fallback_name):
    cur.execute('SELECT id FROM "Institution" WHERE email = %s', (email,))
    row = cur.fetchone()
    if row:
        return row[0]
    iid = uuid.uuid4().hex
    cur.execute(
        'INSERT INTO "Institution" (id, name, email) VALUES (%s, %s, %s) '
        'ON CONFLICT ("name") DO UPDATE SET email = EXCLUDED.email RETURNING id',
        (iid, fallback_name, email),
    )
    return cur.fetchone()[0]

def upsert_deposit(inst_id, qdate, term, term_sort, currency, bid, ask, note=None):
    cur.execute(
        '''INSERT INTO "DepositQuote"
           (id, quote_date, institution_id, term, term_sort, currency, bid_rate, ask_rate, note)
           VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
           ON CONFLICT (institution_id, quote_date, term, currency) DO UPDATE
           SET bid_rate = EXCLUDED.bid_rate, ask_rate = EXCLUDED.ask_rate, note = EXCLUDED.note''',
        (uuid.uuid4().hex, qdate, inst_id, term, term_sort, currency, bid, ask, note),
    )

conn.commit()
```

## 6. 数据可见性

入库后立即可见：

- 前台 `http://<site>/quotes`（默认展示最新报价日，可按日期/机构筛选）
- admin 后台 Institutions / Quotes 管理页（可人工修正）
- AI 聊天 `/chat`（agent 通过工具读取最新/指定日期报价）

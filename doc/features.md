# 功能说明

## 1. 项目级能力

| 功能 | 说明 |
|------|------|
| SEO 前台站点 | SSR 输出可索引 HTML，支持 Open Graph 等 meta |
| Admin 仪表盘 | 独立进程/域名的内容与商品管理后台 |
| Agent（AI-native） | crate `agent` + Rig：`/agent/chat`；OpenAI Agent；Cornucopia 读已发布 Post 与 OTC 报价 |
| CLI | `settings-init`、`user-add`（及 `migrate` 占位） |
| Ops 脚本 | 构建、上传、运行 site / admin / cli；备份与 systemd 示例 |
| 本地 scripts | 生成 DB 客户端、构建编辑器、watch 开发站点/后台 |

## 2. 前台站点（site）

### 页面与内容

- **首页（Quotes 终端页）**：`/` 即 OTC 报价终端 — Deposits/Bonds 标签页、日期（默认最新报价日）与机构筛选；每期限/每债券高亮最优 Bid（绿）/最优 Ask（红）与最窄点差（琥珀），Mid 与 Sprd(bp) 列，顶部统计条（日期/机构数/期限数/条数/当日最窄点差）；彭博终端风格（深色面板、琥珀强调、等宽数字）
- **文章**：按分类 slug + 文章 slug 访问；列表/详情
- **产品**：按产品 slug 访问详情；产品列表
- **定时发布**：`publish_at`；未发布内容前台不可见（404）

### 体验与 SEO

- SSR + SPA hydration（首屏服务端渲染，随后客户端增强）
- Open Graph / meta title & description（文章、产品等实体字段）
- 明暗主题、移动优先布局
- Favicon 等基础站点组件

### 生产加固（可选 feature）

- HTTP 限流（`tower-governor`）
- 响应压缩（`tower-http`）
- 静态资源预压缩（ops + precompress）

## 3. 管理后台（admin）

### 认证与会话

- 登录 / 登出
- 基于 Session 的鉴权；未登录访问受保护路由跳转登录
- 密码 bcrypt 存储（由 CLI 或注册流程创建用户）

### 文章（Post）

- CRUDL（创建、读取、更新、删除、列表）
- 关联分类（`PostCategory`）
- SEO 字段：`meta_title`、`meta_description`、`h1`、`slug`
- `publish_at` 定时发布
- 富文本正文（Content + Slate 编辑器）

### 分类（PostCategory）

- 分类列表与编辑
- `slug` / `name` 及分类级 SEO 字段

### 产品（Product）

- CRUDL
- SEO 字段与 `publish_at`
- **产品图片**
  - 预览与上传
  - 服务端缩放与格式转换
  - 排序（`order`）
  - 主图标记（`is_hero`）
  - 对话框内删除与 alt 更新

### 机构（Institution）

- CRUDL：`name`（唯一）、`short_name`、`email`（报价来源邮箱）、`is_active`
- 删除为确认输入门槛 + 级联删除其全部报价（删除前可看到报价关联提示）

### 报价（Quotes：存款 DepositQuote / 债券 BondQuote）

- 存款报价 CRUDL：日期、机构、期限（term + term_sort 排序）、币种、Bid/Ask 利率、备注
- 债券报价 CRUDL：日期、机构、债券代码/名称、币种、到期日、票面利率、Bid/Ask 收益率与价格、方向、备注
- 列表按日期区间（from/to）与机构筛选
- 唯一键：存款 `UNIQUE(institution_id, quote_date, term, currency)`；债券 `UNIQUE(institution_id, quote_date, bond_code)`（对应 Python 入库脚本的幂等 upsert，见 [ingestion.md](./ingestion.md)）

### 富文本内容（Content）

- Slate.js 结构化编辑（段落、标题、引用、列表、链接、图片等）
- JSON 持久化到 `Content` 表
- 防抖自动保存
- 内容内图片上传；内容更新时清理已移除图片
- 前台由 Rust 将 JSON 渲染为 HTML

### 设置（Settings）

- `site_url`
- `robots_txt`
- `home_text`（首页文案）

### UI 组件

- 表单、输入控件与错误/响应提示
- `datetime-local` 与 chrono 协作
- `RoutingProgress` 路由进度指示
- Favicons
- 响应式布局（含移动端适配）

## 4. CLI（cli）

| 命令 | 功能 |
|------|------|
| `cargo cli settings-init` | 若不存在则创建默认 Settings 记录 |
| `cargo cli user-add <user> <pass>` | 创建后台用户（bcrypt） |
| `cargo cli migrate` | 迁移占位（历史 Prisma 路径已移除） |

## 5. 数据与媒体相关能力

- 唯一用户名、文章/产品 slug 约束（见 `schema.sql` 索引）
- Content 与 Post/Product 一对一关联（内容级联删除）
- 产品图、内容图分表管理
- SSR 访问记录表（路径、UA）供后续分析使用

## 6. 样式与主题

- 基于 Open Props 思路的设计令牌
- 亮色 / 暗色主题
- Mobile First
- SASS 构建，接入 cargo-leptos `style-file`

## 7. 功能边界（当前状态）

项目仍在积极开发中。已知演进方向包括：

- 已从 Prisma ORM 迁至 Cornucopia SQL-first
- 查询客户端曾由 Clorinde 迁回/对齐 Cornucopia 1.x
- 富文本依赖 Slate.js，而非纯 Rust 编辑器
- CLI `migrate` 尚未提供完整迁移流水线；建表以 `schema.sql` 导入为主

更细的版本变更见根目录 [CHANGELOG.md](../CHANGELOG.md)。

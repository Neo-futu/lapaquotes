# 架构说明

## 1. 项目定位

Lapa 是一套基于 **Rust + Leptos + Axum** 的全栈网站起步模板：同时提供面向访客的 **SEO 前台站点（SSR + SPA hydration）** 与面向运营的 **Admin 管理后台**，数据层采用 **PostgreSQL + SQL-first（Cornucopia）**。

目标是用 Rust 的类型安全与可预测性能，构建可扩展的网站与后台服务。

## 2. 总体架构

```
                    ┌─────────────────────────────────────┐
                    │           Browser / CDN             │
                    └──────────────┬──────────────────────┘
                                   │
          ┌────────────────────────┼────────────────────────┐
          ▼                        ▼                        ▼
   ┌─────────────┐          ┌─────────────┐          ┌─────────────┐
   │  site (前台) │          │ admin (后台) │          │   public/   │
   │ Leptos SSR  │          │ Leptos SSR  │          │ 静态资源/图 │
   │ + hydrate   │          │ + hydrate   │          │             │
   │ Axum HTTP   │          │ Axum HTTP   │          │             │
   └──────┬──────┘          └──────┬──────┘          └─────────────┘
          │                        │
          │   shared crates        │
          │  (lib / content /      │
          │   image_config /       │
          │   cornucopia)          │
          └────────────┬───────────┘
                       ▼
              ┌─────────────────┐
              │   PostgreSQL    │
              │ schema.sql +    │
              │ typed queries   │
              └─────────────────┘

          CLI (clap) ──► 初始化 Settings / 创建用户
          slate (React) ──► 构建产物嵌入 Admin 富文本编辑
          agent crate ──► site 挂载 /agent/* 或独立 :4151（AI 对话）
```

两个可独立部署的二进制应用（`site`、`admin`）共享同一数据库与公共库；CLI 用于运维初始化；富文本编辑器以前端子项目形式构建后由 Admin 加载。AI 能力见独立 crate `agent`（[agent.md](./agent.md)）。

## 3. Cargo Workspace 结构

| Crate / 目录 | 角色 |
|--------------|------|
| `site`（根包 `Cargo.toml`） | 前台站点：首页、文章、产品展示；默认端口 `127.0.0.1:4141` |
| `admin` | 管理后台：登录会话、文章/分类/产品/设置/内容编辑 |
| `cli` | 命令行工具：`settings-init`、`user-add`、`migrate`（占位） |
| `agent` | AI-native：`/agent/chat` + [Rig](https://rig.rs) OpenAI Agent；可独立进程 |
| `lib` | 跨应用公共逻辑与错误类型（可选 `leptos` feature） |
| `content` | Slate JSON ↔ HTML 渲染、CDN URL 辅助 |
| `image_config` | 图片尺寸/格式等配置 |
| `cornucopia` | 由 schema + SQL 查询生成的类型安全 DB 客户端 |
| `queries/` | 手写 SQL（Admin / Site 查询分离） |
| `schema.sql` | 数据库表结构单一来源 |
| `slate/` | React + Slate.js + rspack 富文本编辑器 |
| `style/`、`admin/style/` | SASS（Open Props 风格，明暗主题） |
| `public/`、`admin/public/` | 静态资源 |
| `ops/` | 生产构建、上传、systemd / 反向代理示例 |
| `scripts/` | 本地开发辅助脚本 |

根包同时承担 workspace 成员与 `site` 应用定义；`admin`、`cli` 等为独立成员。

## 4. 运行时分层（site / admin）

两端均采用 **cargo-leptos** 的双端编译模式：

| Feature | 含义 |
|---------|------|
| `ssr` | 服务端：Axum、Postgres 连接池、Tower 中间件、图片处理等 |
| `hydrate` | 浏览器端：Wasm + Leptos hydration |
| `prod` | `compression` + `ratelimit` 组合，用于生产构建 |

典型请求路径：

1. Axum 接收 HTTP，经限流 / 压缩 / 静态文件中间件。
2. `leptos_axum` 完成 SSR 渲染并返回 HTML。
3. 浏览器加载 Wasm 包完成 hydration，后续导航可走客户端路由。
4. Server Functions / 服务端代码通过 Cornucopia + `deadpool-postgres` 访问数据库。

Admin 额外叠加：

- `axum_session` + `axum_session_auth` 会话与鉴权
- `bcrypt` 密码哈希
- 受保护路由（未登录跳转 `/login`）

## 5. 数据模型（核心实体）

来源：`schema.sql`。

| 表 | 用途 |
|----|------|
| `User` / `Session` | 后台用户与会话 |
| `Settings` | `site_url`、`robots_txt`、`home_text` 等站点配置 |
| `Content` / `ContentImage` | 富文本 JSON 内容及内容内图片 |
| `PostCategory` / `Post` | 文章分类与文章（`slug`、SEO 字段、`publish_at`） |
| `Product` / `ProductImage` | 产品及图集（`is_hero`、`order`、alt 等） |
| `Institution` / `DepositQuote` / `BondQuote` | OTC 报价：机构 + 存款/债券每日报价（外部 Python 入库，前台 `/quotes` 展示，AI agent 可读；详见 [ingestion.md](./ingestion.md)） |
| `Ssr` | SSR 访问路径与 UA 记录（分析/调试用） |

内容存储为 **Slate 结构化 JSON**（`Content.json`），前台通过 `content` crate 转为 HTML；定时发布由 `publish_at` 控制，未到发布时间的资源对前台表现为不可见（404）。

## 6. 前台与后台路由概览

**Site（前台）**

- `/` — OTC 报价终端（首页即 Quotes；存款/债券标签页、日期与机构筛选、最优价/最窄点差高亮）
- `/product/:slug` — 产品详情
- `/:category_slug/:slug` — 分类下文章详情

**Admin（后台，需登录）**

- `/login` — 登录
- `/` — 仪表盘首页
- `/settings` — 站点设置
- `/institution` — 机构 CRUDL
- `/quote/deposit`、`/quote/bond` — 存款/债券报价 CRUDL（日期区间与机构筛选）
- `/product`、`/product/...` — 产品 CRUDL 与图片管理
- 文章 / 分类相关路由（列表、创建、编辑、删除）
- 内容编辑器集成（Slate）与图片上传

## 7. 图片与内容管线

1. Admin 上传图片（产品图 / 内容图）。
2. 服务端使用 `image` crate 等做缩放、格式转换（相关目录：`image_upload/`、`image_convert/`）。
3. 元数据写入 `ProductImage` / `ContentImage`；文件落盘到静态目录供站点与 CDN 使用。
4. 富文本中的图片块与内容更新联动（上传与清理移除的图片）。

## 8. 部署形态

`ops/` 提供示例流程：

- 交叉编译目标（见 `.env.example` 中 `TARGET`）
- `site-deploy` / `admin-deploy` / `cli-deploy`
- systemd unit（`lapa-web.service`、`lapa-admin.service`）
- 反向代理配置示例（`ferron.kdl`）
- 静态资源预压缩（precompress）

前台与后台可使用不同域名（如 `SITE_DOMAIN` / `ADMIN_DOMAIN`）分别对外暴露。

## 9. 设计要点小结

- **双应用、单库**：站点与后台进程分离，数据与查询契约共享。
- **SQL-first**：schema + 手写 queries → Cornucopia 生成类型安全调用，替代早期 Prisma / Clorinde 路径。
- **SSR 优先 SEO**：Open Graph、meta、可索引 HTML；再 hydration 增强交互。
- **富文本借前端生态**：Rust 侧负责存储与 HTML 渲染，编辑体验由 Slate.js 承担。
- **生产能力可选开启**：限流、HTTP 压缩、预压缩静态资源。

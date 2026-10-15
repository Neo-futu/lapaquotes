# 技术平台

## 1. 语言与工具链

| 层级 | 选型 | 说明 |
|------|------|------|
| 语言 | Rust（edition 2024） | Workspace 全栈主语言 |
| 工具链 | nightly（`rust-toolchain.toml`） | Leptos nightly features |
| 包管理 | Cargo Workspace | 根包为 `site`，成员含 `admin`、`cli`、`lib` 等 |
| 前端构建（站点/后台 UI） | [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) | SSR + Wasm hydrate、样式与静态资源 |
| 编辑器前端 | Node.js + npm + [Rspack](https://rspack.dev) | `slate/` 子项目 |
| 数据库 | PostgreSQL | 连接串环境变量 `PG__URL` |
| 查询代码生成 | [Cornucopia](https://github.com/cornucopia-rs/cornucopia) | 需 Docker/Podman；输入 `schema.sql` + `queries/*.sql` |

## 2. Web 框架栈

### 前端 UI：Leptos 0.8

- 响应式信号驱动 UI
- SSR + SPA hydration
- `leptos_router` 路由、`leptos_meta` 元信息（Title、Open Graph 等）
- Admin / Site 均为 Leptos 应用，编译为原生服务端 + `cdylib` Wasm 客户端

### 后端 HTTP：Axum 0.8

- 基于 Tokio / Tower / Hyper
- 通过 `leptos_axum` 与 Leptos SSR 集成
- 静态文件服务、`tower-http` 追踪与可选压缩

### 中间件与生产特性

| 能力 | 依赖 / Feature |
|------|----------------|
| 限流 | `tower_governor`，feature `ratelimit` |
| 压缩 | `tower-http` compression，feature `compression` |
| 生产组合 | feature `prod` = compression + ratelimit |
| 预压缩静态资源 | 外部工具 [precompress](https://github.com/ryanfowler/precompress)，见 `ops/` |

## 3. 数据访问

```
schema.sql  +  queries/*.sql
        │
        ▼
   cornucopia CLI
        │
        ▼
 cornucopia/  (生成的 Rust 查询 API)
        │
        ▼
 deadpool-postgres 连接池  →  PostgreSQL
```

- **SQL-first**：业务 SQL 手写在 `queries/`，按场景拆分（如 `post.sql`、`admin_post.sql`）。
- 运行时使用 `deadpool-postgres` + `tokio-postgres`。
- 配置通过 `config` crate + 环境变量（`PG__` 前缀）加载；本地常用 `.env`（`dotenvy`）。

## 4. 认证与安全（Admin）

| 组件 | 用途 |
|------|------|
| `axum_session` | 会话存储（表 `Session`） |
| `axum_session_auth` | 认证集成 |
| `bcrypt` | 用户密码哈希 |
| 受保护路由 | 未登录访问后台页面时重定向登录 |

CLI `user-add` 写入带 bcrypt 哈希的用户；`settings-init` 初始化唯一 Settings 行。

## 5. 富文本与内容

| 部分 | 技术 |
|------|------|
| 编辑器 UI | React 19 + Slate.js（`slate` / `slate-react` / `slate-history`） |
| 打包 | Rspack |
| 存储 | Postgres `Content.json`（Slate JSON） |
| 渲染 | Rust `content` crate：JSON → HTML（段落、标题、列表、链接、图片等） |
| 体验 | 防抖自动保存（`use-debounce`） |

因 Rust 生态尚无成熟富文本编辑器，编辑侧采用 JS，渲染与存储侧保持 Rust 类型与性能优势。

## 6. 图片处理

- Rust `image` crate：上传后缩放、格式转换
- `kamadak-exif`（Admin）：处理 EXIF（如方向校正相关场景）
- `image_config`：统一尺寸/产物配置
- 产品图支持预览、排序、`is_hero`、alt 编辑与删除

## 7. 样式与体验

- **SASS**（`style/main.scss` 等），曾探索原生 CSS + lightningcss，当前以 SASS 为主
- 设计基础参考 [Open Props](https://open-props.style)
- 明暗主题、Mobile First
- 使用 `@custom-media`、`@container` 等现代 CSS 能力（经 SASS 管线）

## 8. CLI 与运维

| 工具 | 说明 |
|------|------|
| `clap` | CLI 子命令解析 |
| `cuid2` | 实体 ID 生成 |
| `chrono` | 时间（含 `datetime-local` 表单） |
| `tracing` / `tracing-subscriber` | 结构化日志（`RUST_LOG`） |
| `ops/*.sh` | 构建、上传、备份、systemd、反向代理示例 |
| `scripts/*.sh` | 本地：生成 DB 客户端、构建 Slate、watch 开发 |

## 9. 关键依赖版本快照（Workspace）

以根 `Cargo.toml` `[workspace.dependencies]` 为准，主要包括：

- `leptos` / `leptos_axum` / `leptos_meta` / `leptos_router` **0.8**
- `axum` **0.8**
- `tokio` **1.x**
- `tower` **0.5**、`tower-http` **0.6**、`tower_governor` **0.7**
- `serde` / `serde_json`、`thiserror` **2**、`chrono`、`config` **0.15**

Slate 子项目见 `slate/package.json`（React 19、Slate ~0.112、Rspack 1.x）。

## 10. 本地开发最小栈

1. Rust nightly + `cargo-leptos`
2. PostgreSQL，导入 `schema.sql`
3. `cornucopia` + Docker/Podman 生成客户端
4. Node.js（构建 `slate/`）
5. `.env` 中配置 `PG__URL`

详见根目录 README「Run」与 [`scripts/README.md`](../scripts/README.md)。

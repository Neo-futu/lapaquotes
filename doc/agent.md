# Agent / AI-native

可插拔 LLM 后端：`agent::providers`（后续可继续加 Claude / 本地模型等）。

## CURSOR vs OpenRouter：查库怎么走？

本项目**没有** Cursor Skill，也**没有**接 MCP。工具是 Rust 里的 **Rig `DynamicTool`**。

| | OpenRouter / OpenAI（Rig） | Cursor Cloud Agents |
|--|---------------------------|---------------------|
| Key | `OPENAI_API_KEY` + `OPENAI_BASE_URL` | `CURSOR_API_KEY`（`crsr_…`） |
| 查库 | **是**：模型发 `tool_call` → 我们的 Rust 闭包查 Postgres → 结果回给模型 | **不是 tools**：服务端先查库，把 LIVE_DATA 写进 prompt |
| 速度 | 通常更快 | 建 agent + 轮询，较慢 |
| 代码 | `quote_tools.rs` / `tools.rs` / `rig_agent.rs` | `providers/cursor.rs` + `live_quotes_snapshot` |

### 官方 Rig 怎么调「模型 + tool + 库」（本项目对齐）

对照 [0xPlaygrounds/rig examples](https://github.com/0xPlaygrounds/rig/tree/main/examples)：

| 官方示例 | 本项目 |
|----------|--------|
| `agent_with_tools` | `DynamicTool` in `quote_tools.rs` / `tools.rs` |
| `agent_with_context` | 预取 `recent_quote_dates` 注入 `.context(...)` |
| `force_tool_first_turn` | `hooks::ForceToolOnFirstTurn`（仅第 1 轮 `tool_choice=Required`） |
| `multi_turn_agent` | `agent.prompt(...).max_turns(N)` |

```
用户问题
  → AgentBuilder(model + preamble + dynamic_tools + context)
  → add_hook(ForceToolOnFirstTurn)
  → turn 1: LLM（必须 tool_call）→ Rust 闭包查 Postgres
  → turn 2: LLM 读 tool 结果 → 自然语言答案
```

日志里的 `invoke_agent` / `Turns: 1/4` **就是**官方多轮 Agent 循环：每一 turn = 一次大模型 HTTP，不是「少了 tool」。  
库不在模型里；库在 **tool 闭包**（`deposit_by_date` 等）里。

Cursor 输入输出：

```
用户问题 → 服务端拼 LIVE_DATA（已查库）→ Cursor 只读文本回答
```

## 架构

```
POST /agent/chat  { question, provider?, model?, history? }
        ↓
AgentService → providers::{openai|cursor}
        ├── openai/openrouter → Rig Agent + DynamicTools（查库）
        └── cursor → Cloud Agents + LIVE_DATA 快照
```

| 模块 | 职责 |
|------|------|
| `agent::api` | HTTP 契约 |
| `agent::providers` | 多模型入口（`openai` / `cursor`） |
| `agent::providers::openai` | Rig + OpenAI 兼容 |
| `agent::providers::cursor` | Cursor Cloud Agents |
| `agent::hooks` | `ForceToolOnFirstTurn`（官方 force_tool_first_turn） |
| `agent::tools` / `quote_tools` | Rig DynamicTool（仅 openai 路径） |
| `agent::mcp` | Rig MCP 客户端配置/状态（`feature = mcp` → `rig/rmcp`） |
| `agent::channels::feishu` | 飞书机器人 Webhook → `AgentService` |
| `agent::channels::whatsapp` | WhatsApp Cloud API Webhook → `AgentService` |
| `agent::bi` | BI 层：平台报价 → `Bi*` 表 → 飞书多维表格 |
| `agent::context` / `quote_context` | Cornucopia 读库 |
| `agent::rig_agent` | AgentBuilder 装配 |
| `agent::service` | 编排 → providers |
| `agent::router` | `/health`、`/chat`、`/mcp/status`、`/channels/*`、`/bi/*` |

## Rig 支持 MCP 吗？飞书怎么接？

**支持。** Rig 官方通过 feature `rmcp`（crate `rig-rmcp`）把远程 MCP server 的工具挂到 Agent 上，和本地 `DynamicTool` 一样参与多轮 tool 循环。

```sh
cargo check -p agent --features mcp
GET /agent/mcp/status   # 查看 MCP 配置是否就绪
```

**飞书机器人 ≠ MCP。** 飞书用开放平台事件订阅（HTTP Webhook），配置：

1. 创建飞书应用，开通机器人能力与 `im:message` 等权限  
2. 事件订阅请求地址：`https://<公网域名>/agent/channels/feishu/events`  
3. 订阅 `im.message.receive_v1`  
4. `.env` 填写 `FEISHU_APP_ID` / `FEISHU_APP_SECRET` / `FEISHU_VERIFICATION_TOKEN`  
5. 用户发消息 → 本服务 `AgentService::chat`（与网页 Chat 同源）→ 飞书 reply  

暂不支持事件加密（Encrypt Key）；调试时可先清空加密密钥。

## WhatsApp（Meta Cloud API）

无飞书权限时可用 WhatsApp 对话（与网页 Chat 同源 `AgentService`）。

1. [Meta for Developers](https://developers.facebook.com/) 创建应用，添加 **WhatsApp** 产品  
2. 获取 **Phone number ID** 与 **Access token**  
3. Webhook Callback URL：`https://<公网域名>/agent/channels/whatsapp/webhook`  
4. Verify token：自定义字符串，写入 `.env` 的 `WHATSAPP_VERIFY_TOKEN`  
5. 订阅字段勾选 **messages**  
6. `.env`：

```env
WHATSAPP_ACCESS_TOKEN=...
WHATSAPP_PHONE_NUMBER_ID=...
WHATSAPP_VERIFY_TOKEN=your-secret
# 可选：只允许自己的号码（避免被陌生人刷）
WHATSAPP_ALLOWED_FROM=85291234567
```

本地开发需公网 HTTPS（如 ngrok / cloudflared）指向站点（默认 nest 在 `/agent`）。  
测试号需在 Meta 控制台把你的手机号加入允许列表，再用 WhatsApp 发文字即可。

## BI 层与飞书多维表格

分层（`agent/src/bi`）：

```text
Platform DB (DepositQuote / BondQuote)
    → extract → BiQuoteFact
    → BiService
    → bitable adapter（发现 TABLE_ID / VIEW_ID / NAME）
    → Feishu Base
```

**BI 库表**（见 `schema.sql`）：`BiBitableTable` / `BiBitableView` / `BiBitableField` / `BiQuoteFact` / `BiSyncJob`。

**Env 仅需三项**（不要写死 TABLE_ID）：

```env
FEISHU_BITABLE_ACCESS_TOKEN=pt-...
FEISHU_BITABLE_APP_TOKEN=N4npbXv29ah1MUs0abycR3mtnnb
FEISHU_BITABLE_WEB_HOST=https://futu.feishu.cn
FEISHU_BITABLE_API_BASE=https://base-api.feishu.cn
```

`pt-` 必须走 `base-api.feishu.cn`。表/视图/字段由 `POST /agent/bi/catalog/refresh` 拉取后落库；`migrate` 会把 Base 内全部模板表改成：

| role | 表名 | 用途 |
|------|------|------|
| `quote_ledger` | OTC报价台账 | 逐条存款/债券 |
| `quote_digest` | 最新报价推送 | 按机构摘要（原消息推送表） |

```sh
python scripts/bi/apply_schema.py
cargo run -p agent --example bi_sync --features standalone -- migrate
cargo run -p agent --example bi_sync --features standalone -- verify
# 日更（优先打 HTTP，失败再 cargo）
python scripts/bi/daily_sync.py
```

HTTP：`/agent/bi/catalog`、`/migrate`、`/sync`、`/verify`、`/facts`、`/jobs`、`/dashboards`。

完整架构、日更与仪表盘说明见 **[doc/bi.md](bi.md)**。

## 环境变量

| 变量 | 说明 |
|------|------|
| `PG__URL` | Postgres |
| `LLM_PROVIDER` | `auto` / `openai` / `openrouter` / `cursor`。`auto`：有 `CURSOR_API_KEY` 优先 Cursor，否则 OpenRouter。Chat 默认 Cursor，可再切换 |
| OpenRouter | `OPENAI_API_KEY=sk-or-v1-…` + `OPENAI_BASE_URL=https://openrouter.ai/api/v1`；默认模型 `deepseek/deepseek-chat`（Chat 可改） |
| `CURSOR_API_KEY` | Cursor Cloud Agents（`crsr_…`） |
| `CURSOR_MODEL` | 默认 `auto-smart` |
| `OPENAI_API_KEY` | OpenAI 兼容（`sk-…`）。**不要**填 `crsr_` |
| `OPENAI_BASE_URL` | 可选；未用时勿写成空字符串 |
| `OPENAI_MODEL` | 默认 `deepseek/deepseek-chat`（Chat 可改） |
| `AGENT_MAX_TURNS` | Rig 多轮上限，默认 `4`（每轮一次 LLM HTTP） |
| `AGENT_MAX_TOKENS` | completion max tokens，默认 `4096` |
| `AGENT_ADDR` | 独立服务，默认 `127.0.0.1:4151` |

验证 Cursor 路径：

```sh
cargo run -p agent --example cursor_hello
```

## 前端聊天页

浏览器打开：**http://127.0.0.1:4141/chat**（导航栏 Chat）。

常见 Agent 聊天窗口：欢迎语 + 建议芯片、气泡线程、打字指示、Enter 发送 / Shift+Enter 换行、新对话。经 Leptos Server Function 调用同一套 `AgentService`。

## API

### 按 slug 问文章（openai 路径会注入 context + tools）

```json
POST /agent/chat
{
  "question": "这篇文章的核心观点是什么？",
  "slug": "hello-world"
}
```

### 多轮

```json
{
  "question": "那摘要再缩短成三句话",
  "history": [
    { "role": "user", "content": "这篇文章讲什么？" },
    { "role": "assistant", "content": "……" }
  ]
}
```

响应带回更新后的 `history`。

### 独立进程

```sh
cargo run -p agent --features standalone --bin agent
```

## 扩展更多大模型

在 `agent/src/providers/` 新增模块（例如 `anthropic.rs`），在 `ProviderKind` / `resolve_provider` / `chat` 分发即可；HTTP 契约与 Chat UI 不用改。

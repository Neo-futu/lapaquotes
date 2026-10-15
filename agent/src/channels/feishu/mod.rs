//! 飞书（Lark）机器人通道。
//!
//! 在飞书开放平台配置事件订阅 URL：
//! `https://<host>/agent/channels/feishu/events`
//!
//! 需要环境变量：
//! - `FEISHU_APP_ID` / `FEISHU_APP_SECRET` — 发消息用 tenant_access_token
//! - `FEISHU_VERIFICATION_TOKEN` — 校验事件（可选但建议）
//!
//! 流程：url_verification → 收 `im.message.receive_v1` → 后台跑 Agent → reply。

pub mod api;
mod webhook;

pub use api::{FeishuConfig, tenant_access_token};
pub use webhook::{feishu_router, handle_event};

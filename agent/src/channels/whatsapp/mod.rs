//! WhatsApp Cloud API（Meta）通道。
//!
//! 在 [Meta for Developers](https://developers.facebook.com/) 创建应用 →
//! WhatsApp → 配置 Webhook：
//!
//! - Callback URL: `https://<host>/agent/channels/whatsapp/webhook`
//! - Verify token: 与 `.env` 中 `WHATSAPP_VERIFY_TOKEN` 一致
//! - 订阅字段: `messages`
//!
//! 环境变量：
//! - `WHATSAPP_ACCESS_TOKEN` — 系统用户 / 临时访问令牌
//! - `WHATSAPP_PHONE_NUMBER_ID` — 发送用的电话号码 ID
//! - `WHATSAPP_VERIFY_TOKEN` — Webhook 校验口令（自定义字符串）
//! - `WHATSAPP_API_VERSION` — 默认 `v21.0`
//! - `WHATSAPP_ALLOWED_FROM` — 可选，逗号分隔白名单（E.164 数字，无 +）

mod api;
mod webhook;

pub use api::WhatsAppConfig;
pub use webhook::whatsapp_router;

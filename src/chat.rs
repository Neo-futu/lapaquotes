use leptos::html::Div;
use leptos::prelude::*;
use leptos_meta::Title;
use serde::{Deserialize, Serialize};

use crate::util::AlertDanger;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatReply {
    pub msg: String,
    pub slug: Option<String>,
    pub history: Vec<ChatTurn>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatProviderInfo {
    pub provider: String,
    pub hint: String,
    pub openrouter_ready: bool,
    pub cursor_ready: bool,
    pub model: String,
}

/// Cheap OpenRouter models that support tools (verified in-region).
const MODEL_OPTIONS: &[(&str, &str)] = &[
    ("deepseek/deepseek-chat", "DeepSeek Chat"),
    ("mistralai/mistral-nemo", "Mistral Nemo"),
    ("qwen/qwen-2.5-7b-instruct", "Qwen 2.5 7B"),
];

const SUGGESTIONS: &[&str] = &[
    "最新日期的存款报价，给一家机构和价格",
    "最近有哪些存款报价？",
    "列出近期有报价的日期",
];

#[component]
pub fn ChatPage() -> impl IntoView {
    let question = RwSignal::new(String::new());
    let history = RwSignal::new(Vec::<ChatTurn>::new());
    let error = RwSignal::new(Option::<String>::None);
    let source = RwSignal::new(String::new());
    let backend = RwSignal::new(String::from("cursor"));
    let model = RwSignal::new(String::from("deepseek/deepseek-chat"));
    let thread_el = NodeRef::<Div>::new();

    let send = ServerAction::<SendChat>::new();
    let pending = send.pending();
    let value = send.value();

    let provider = Resource::new(|| (), |_| async move { chat_provider_info().await });

    Effect::new(move |_| {
        if let Some(Ok(info)) = provider.get() {
            // Default Cursor; fall back to OpenRouter only if Cursor key missing.
            if info.cursor_ready {
                backend.set("cursor".into());
            } else if info.openrouter_ready {
                backend.set("openai".into());
            }
            if !info.model.trim().is_empty() {
                model.set(info.model);
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = value.get() {
            match result {
                Ok(reply) => {
                    history.set(reply.history);
                    source.set(reply.source);
                    question.set(String::new());
                    error.set(None);
                }
                Err(e) => error.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        let _ = history.get();
        let _ = pending.get();
        if let Some(el) = thread_el.get() {
            let _ = el.set_scroll_top(el.scroll_height());
        }
    });

    let dispatch_send = move || {
        if pending.get() {
            return;
        }
        let q_trim = question.get().trim().to_string();
        if q_trim.is_empty() {
            return;
        }
        let prior = history.get();
        let history_json = serde_json::to_string(&prior).unwrap_or_else(|_| "[]".into());
        let mut next = prior;
        next.push(ChatTurn {
            role: "user".into(),
            content: q_trim.clone(),
        });
        history.set(next);
        question.set(String::new());
        error.set(None);

        send.dispatch(SendChat {
            question: q_trim,
            slug: String::new(),
            history_json,
            provider: backend.get(),
            model: model.get(),
        });
    };

    let on_clear = move |_| {
        history.set(Vec::new());
        error.set(None);
        source.set(String::new());
        send.clear();
    };

    let use_suggestion = move |text: &'static str| {
        move |_| {
            if pending.get() {
                return;
            }
            let prior = history.get();
            let history_json = serde_json::to_string(&prior).unwrap_or_else(|_| "[]".into());
            let mut next = prior;
            next.push(ChatTurn {
                role: "user".into(),
                content: text.to_string(),
            });
            history.set(next);
            question.set(String::new());
            error.set(None);
            send.dispatch(SendChat {
                question: text.to_string(),
                slug: String::new(),
                history_json,
                provider: backend.get(),
                model: model.get(),
            });
        }
    };

    view! {
        <Title text="Chat" />
        <section class="GptChat">
            <header class="GptChat-top">
                <div class="GptChat-topLeft">
                    <select
                        class="GptChat-select"
                        prop:value=move || backend.get()
                        prop:disabled=move || pending.get()
                        on:change=move |ev| backend.set(event_target_value(&ev))
                    >
                        <Suspense fallback=|| view! {
                            <option value="openai">"OpenRouter"</option>
                        }>
                            {move || {
                                provider.get().map(|res| {
                                    let (or_ok, cu_ok) = match res {
                                        Ok(i) => (i.openrouter_ready, i.cursor_ready),
                                        Err(_) => (true, true),
                                    };
                                    view! {
                                        <option value="openai" prop:disabled=!or_ok>
                                            "OpenRouter"
                                        </option>
                                        <option value="cursor" prop:disabled=!cu_ok>
                                            "Cursor"
                                        </option>
                                    }
                                })
                            }}
                        </Suspense>
                    </select>
                    <Show when=move || backend.get() == "openai">
                        <select
                            class="GptChat-select"
                            prop:value=move || model.get()
                            prop:disabled=move || pending.get()
                            on:change=move |ev| model.set(event_target_value(&ev))
                        >
                            {MODEL_OPTIONS
                                .iter()
                                .copied()
                                .map(|(id, label)| {
                                    view! {
                                        <option value=id>{label}</option>
                                    }
                                })
                                .collect_view()}
                        </select>
                    </Show>
                </div>
                <button
                    type="button"
                    class="GptChat-new"
                    on:click=on_clear
                    prop:disabled=move || pending.get() || history.get().is_empty()
                >
                    "新聊天"
                </button>
            </header>

            <div class="GptChat-scroll" role="log" aria-live="polite" node_ref=thread_el>
                <div class="GptChat-inner">
                    <Show
                        when=move || !history.get().is_empty()
                        fallback=move || {
                            view! {
                                <div class="GptChat-hero">
                                    <h1>"有什么可以帮忙的？"</h1>
                                    <p class="GptChat-heroSub">
                                        "可询问存款/债券报价，或已发布文章相关问题。"
                                    </p>
                                    <div class="GptChat-prompts">
                                        {SUGGESTIONS
                                            .iter()
                                            .copied()
                                            .map(|s| {
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="GptChat-prompt"
                                                        on:click=use_suggestion(s)
                                                    >
                                                        {s}
                                                    </button>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                </div>
                            }
                        }
                    >
                        <For
                            each=move || {
                                history
                                    .get()
                                    .into_iter()
                                    .enumerate()
                                    .collect::<Vec<_>>()
                            }
                            key=|(i, turn)| format!("{i}:{}:{}", turn.role, turn.content.len())
                            children=move |(_i, turn)| {
                                let is_user = turn.role == "user";
                                let row = if is_user {
                                    "GptChat-msg GptChat-msg--user"
                                } else {
                                    "GptChat-msg GptChat-msg--assistant"
                                };
                                view! {
                                    <div class=row>
                                        <div class="GptChat-msgInner">
                                            <Show when=move || !is_user>
                                                <span class="GptChat-icon" aria-hidden="true">"✦"</span>
                                            </Show>
                                            <div class="GptChat-text">{turn.content}</div>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </Show>
                    <Show when=move || pending.get()>
                        <div class="GptChat-msg GptChat-msg--assistant">
                            <div class="GptChat-msgInner">
                                <span class="GptChat-icon" aria-hidden="true">"✦"</span>
                                <div class="GptChat-thinking">
                                    <span></span><span></span><span></span>
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>
            </div>

            <div class="GptChat-bottom">
                <div class="GptChat-bottomInner">
                    <Show when=move || error.get().is_some()>
                        {move || {
                            error
                                .get()
                                .map(|text| view! { <AlertDanger text /> }.into_any())
                                .unwrap_or_else(|| view! { <></> }.into_any())
                        }}
                    </Show>
                    <Show when=move || !source.get().is_empty() && !pending.get()>
                        <p class="GptChat-meta">
                            {move || format!("来源 {} · {}", source.get(), model.get())}
                        </p>
                    </Show>
                    <form
                        class="GptChat-form"
                        on:submit=move |ev| {
                            ev.prevent_default();
                            dispatch_send();
                        }
                    >
                        <textarea
                            class="GptChat-input"
                            rows="1"
                            required
                            placeholder="询问任何问题…"
                            prop:value=move || question.get()
                            on:input=move |ev| question.set(event_target_value(&ev))
                            on:keydown=move |ev| {
                                if ev.key() == "Enter" && !ev.shift_key() {
                                    ev.prevent_default();
                                    dispatch_send();
                                }
                            }
                            prop:disabled=move || pending.get()
                        ></textarea>
                        <button
                            type="submit"
                            class="GptChat-send"
                            prop:disabled=move || pending.get() || question.get().trim().is_empty()
                            aria-label="发送"
                        >
                            {move || if pending.get() { "…" } else { "↑" }}
                        </button>
                    </form>
                    <p class="GptChat-footnote">
                        "Enter 发送 · Shift+Enter 换行"
                    </p>
                </div>
            </div>
        </section>
    }
}

#[server(ChatProviderInfoFn, "/api")]
pub async fn chat_provider_info() -> Result<ChatProviderInfo, ServerFnError> {
    let kind = agent::resolve_provider().map_err(ServerFnError::new)?;
    let openrouter_ready = std::env::var("OPENAI_API_KEY")
        .ok()
        .map(|s| {
            let t = s.trim();
            !t.is_empty() && !t.starts_with("crsr_")
        })
        .unwrap_or(false);
    let cursor_ready = std::env::var("CURSOR_API_KEY")
        .ok()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let model =
        std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "deepseek/deepseek-chat".into());
    Ok(ChatProviderInfo {
        provider: agent::provider_label(kind),
        hint: agent::provider_ui_hint(kind).to_string(),
        openrouter_ready,
        cursor_ready,
        model,
    })
}

#[server(SendChat, "/api")]
pub async fn send_chat(
    question: String,
    slug: String,
    history_json: String,
    provider: String,
    model: String,
) -> Result<ChatReply, ServerFnError> {
    let history: Vec<ChatTurn> =
        serde_json::from_str(&history_json).unwrap_or_default();

    let slug = {
        let t = slug.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    };

    let provider = {
        let t = provider.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    };

    let model = {
        let t = model.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    };

    let pool = crate::server::db::use_pool()?;
    let service = agent::AgentService::new(pool);
    let res = service
        .chat(agent::ChatRequest {
            question: question.trim().to_string(),
            slug,
            post_id: None,
            provider,
            model,
            history: history
                .into_iter()
                .map(|t| agent::ChatTurn {
                    role: t.role,
                    content: t.content,
                })
                .collect(),
        })
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(ChatReply {
        msg: res.msg,
        slug: res.slug,
        source: res.source,
        history: res
            .history
            .into_iter()
            .map(|t| ChatTurn {
                role: t.role,
                content: t.content,
            })
            .collect(),
    })
}

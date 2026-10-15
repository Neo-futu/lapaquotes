//! Verify OpenRouter key + cheap model:
//!   cargo run -p agent --example openrouter_hello
//!
//! Reads OPENAI_API_KEY / OPENAI_BASE_URL / OPENAI_MODEL from .env

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let key = std::env::var("OPENAI_API_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .expect("OPENAI_API_KEY required");
    let base = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".into());
    let model =
        std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "mistralai/mistral-nemo".into());

    let url = format!(
        "{}/chat/completions",
        base.trim_end_matches('/')
    );
    let body = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": "Reply with exactly: OK" }],
        "max_tokens": 16
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .bearer_auth(&key)
        .header("HTTP-Referer", "http://127.0.0.1:4141")
        .header("X-Title", "LapaQuotes")
        .json(&body)
        .send()
        .await
        .expect("request failed");
    let status = resp.status();
    let text = resp.text().await.expect("read body");
    if !status.is_success() {
        panic!("HTTP {status}: {text}");
    }
    let v: serde_json::Value = serde_json::from_str(&text).expect("json");
    let reply = v["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("(empty)");
    let used = v["model"].as_str().unwrap_or(&model);
    println!("OK model={used} reply={reply}");
}

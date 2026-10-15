//! Verify CURSOR_API_KEY chat path: cargo run -p agent --example cursor_hello
use agent::providers::cursor::{api_key_from_env, chat_with_cursor};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let key = api_key_from_env().expect("CURSOR_API_KEY (crsr_…) required in .env");
    let reply = chat_with_cursor(&key, "你好", &[])
        .await
        .expect("cursor chat failed");
    println!("OK reply={reply}");
}

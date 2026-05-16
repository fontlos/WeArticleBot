use bytes::Bytes;
use lark::WebSocketClient;
use lark::data::EventEnvelope;

use std::env;

fn init_log() {
    use logforth::append;
    use logforth::layout::TextLayout;
    use logforth::record::Level;
    use logforth::record::LevelFilter;

    logforth::starter_log::builder()
        .dispatch(|d| {
            d.filter(LevelFilter::MoreSevereEqual(Level::Debug))
                .append(append::Stdout::default().with_layout(TextLayout::default()))
        })
        .apply();
}

#[tokio::main]
async fn main() {
    init_log();

    dotenvy::dotenv().ok();
    let app_id = env::var("APP_ID").unwrap();
    let app_secret = env::var("APP_SECRET").unwrap();

    let mut websocket = WebSocketClient::connect(&app_id, &app_secret)
        .await
        .expect("Failed to initialize Lark bot");

    while let Some(event) = websocket.recv().await {
        tokio::spawn(async move {
            handle_event(event).await;
        });
    }
}

async fn handle_event(event: Bytes) {
    let app_id = env::var("APP_ID").unwrap();
    let app_secret = env::var("APP_SECRET").unwrap();

    let session = lark::Session::new(&app_id, &app_secret);
    let envelope: EventEnvelope = match serde_json::from_slice(&event) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("解析事件信封失败: {}", e);
            return;
        }
    };

    println!(
        "收到事件: {} (event_id: {})",
        envelope.header.event_type, envelope.header.event_id
    );

    // 根据 event_type 分发
    match envelope.header.event_type.as_str() {
        "im.message.receive_v1" => {
            let chat_id = envelope.event["message"]["chat_id"].as_str().unwrap_or("");
            let content_str = envelope.event["message"]["content"].as_str().unwrap_or("");

            // 解析 content 获取文本
            let content: serde_json::Value = serde_json::from_str(content_str).unwrap();
            let text = content["text"].as_str().unwrap_or("");

            // 去掉 @ 标记
            let clean_text = text.replace("@_user_1", "").trim().to_string();

            // 调用
            session.reply_to_chat(chat_id, &clean_text).await.unwrap();
        }
        // 其他事件类型...
        _ => {
            println!("未处理的事件类型: {}", envelope.header.event_type);
        }
    }
}

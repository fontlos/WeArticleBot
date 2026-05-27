use bytes::Bytes;
use lark::event::{EventEnvelope, EventType};

pub async fn handle(event: Bytes) {
    println!("Received event: {}", String::from_utf8_lossy(&event));
    let envelope: EventEnvelope = match EventEnvelope::from_bytes(&event) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("解析事件信封失败: {}", e);
            return;
        }
    };

    println!(
        "收到事件: {:?}: {}",
        envelope.event_type(), envelope.event_id()
    );

    // 根据 event_type 分发
    match envelope.event_type() {
        EventType::ImMessageReceive => {
            let chat_id = envelope.event["message"]["chat_id"].as_str().unwrap_or("");
            let content_str = envelope.event["message"]["content"].as_str().unwrap_or("");

            // 解析 content 获取文本
            let content: serde_json::Value = serde_json::from_str(content_str).unwrap();
            let text = content["text"].as_str().unwrap_or("");

            // 去掉 @ 标记
            let clean_text = text.replace("@_user_1", "").trim().to_string();
            let (cmd, _args) = parse_command(&clean_text);
            match cmd {
                "help" => send_help(chat_id).await,
                "login" => send_login_qrcode(chat_id).await,
                _ => {}
            }
        }

        EventType::Unsupported => {
            println!("未处理的事件类型: {:?}", envelope.event_type());
        }
    }
}

/// 解析命令, 错误的命令统统返回帮助信息
pub fn parse_command(input: &str) -> (&str, &str) {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ("help", "");
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let cmd = parts.next().unwrap_or("help");
    let args = parts.next().unwrap_or("");

    (cmd, args)
}

// 如果确定需要 owned String，由调用方自己 .to_string()

async fn send_help(chat_id: &str) {
    let session = crate::lark();
    let help_text = "命令提示
- help: 显示帮助信息
- login: 获取微信登录二维码";
    session.send_text_to_chat(chat_id, help_text).await.unwrap();
}

async fn send_login_qrcode(chat_id: &str) {
    let lark = crate::lark();
    let wechat = crate::wechat();

    lark.send_text_to_chat(chat_id, "正在获取登陆二维码")
        .await
        .unwrap();

    wechat.create_session().await.unwrap();
    let qrcode_bytes = wechat.get_qrcode().await.unwrap();
    let image_key = lark.upload_image(&qrcode_bytes).await.unwrap();
    lark.send_image_to_chat(chat_id, &image_key).await.unwrap();
}

//! 全局上下文

use std::env;
use std::sync::OnceLock;

use lark::WebSocketClient;

static LARK: OnceLock<lark::Session> = OnceLock::new();
static WECHAT: OnceLock<wechat::Session> = OnceLock::new();

/// 初始化全局上下文并建立飞书长连接, 只能调用一次
/// TODO: 长连接建立的最后一步应该移动到外面
pub async fn init() -> WebSocketClient {
    dotenvy::dotenv().ok();
    let app_id = env::var("APP_ID").expect("APP_ID not set");
    let app_secret = env::var("APP_SECRET").expect("APP_SECRET not set");

    let cookie = std::fs::File::open("cookies.json").expect("failed to open cookies.json");
    let buffer = std::io::BufReader::new(cookie);

    let lark = lark::Session::new(&app_id, &app_secret);
    let wechat = wechat::Session::load(buffer).expect("failed to load wechat session");

    assert!(LARK.set(lark).is_ok(), "lark session already initialized");
    assert!(WECHAT.set(wechat).is_ok(), "wechat session already initialized");

    // TODO: 将 connect 方法独立出去, 使这里可以成为同步初始化函数
    WebSocketClient::connect(&app_id, &app_secret)
        .await
        .expect("Failed to initialize Lark bot")
}

/// 获取飞书会话
pub fn lark() -> &'static lark::Session {
    LARK.get().expect("context not initialized, call context::init first")
}

/// 获取微信会话
pub fn wechat() -> &'static wechat::Session {
    WECHAT.get().expect("context not initialized, call context::init first")
}

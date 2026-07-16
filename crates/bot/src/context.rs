//! 全局上下文

use std::env;
use std::sync::OnceLock;

use lark::WebSocketClient;

static APP_ID: OnceLock<String> = OnceLock::new();
static APP_SECRET: OnceLock<String> = OnceLock::new();
static LARK: OnceLock<lark::Session> = OnceLock::new();
static WECHAT: OnceLock<wechat::Session> = OnceLock::new();

/// 同步初始化: 环境变量 / cookie / 会话, 只能调用一次
pub fn init() {
    dotenvy::dotenv().ok();
    let app_id = env::var("APP_ID").expect("APP_ID not set");
    let app_secret = env::var("APP_SECRET").expect("APP_SECRET not set");

    let cookie = std::fs::File::open("cookies.json").expect("failed to open cookies.json");
    let buffer = std::io::BufReader::new(cookie);

    let lark = lark::Session::new(&app_id, &app_secret);
    let wechat = wechat::Session::load(buffer).expect("failed to load wechat session");

    assert!(APP_ID.set(app_id).is_ok(), "app_id already initialized");
    assert!(APP_SECRET.set(app_secret).is_ok(), "app_secret already initialized");
    assert!(LARK.set(lark).is_ok(), "lark session already initialized");
    assert!(WECHAT.set(wechat).is_ok(), "wechat session already initialized");
}

/// 建立飞书长连接(需先调用 init)
pub async fn connect() -> WebSocketClient {
    let app_id = APP_ID
        .get()
        .expect("context not initialized, call context::init first");
    let app_secret = APP_SECRET
        .get()
        .expect("context not initialized, call context::init first");
    WebSocketClient::connect(app_id, app_secret)
        .await
        .expect("Failed to initialize Lark bot")
}

/// 获取飞书会话
pub fn lark() -> &'static lark::Session {
    LARK.get()
        .expect("context not initialized, call context::init first")
}

/// 获取微信会话
pub fn wechat() -> &'static wechat::Session {
    WECHAT
        .get()
        .expect("context not initialized, call context::init first")
}

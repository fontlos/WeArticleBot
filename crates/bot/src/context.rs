//! 全局上下文

use std::env;
use std::sync::OnceLock;

static LARK: OnceLock<lark::Session> = OnceLock::new();
static WECHAT: OnceLock<wechat::Session> = OnceLock::new();

pub fn init() {
    let app_id = env::var("APP_ID").expect("APP_ID not set");
    let app_secret = env::var("APP_SECRET").expect("APP_SECRET not set");

    let cookie = std::fs::File::open("cookies.json").expect("failed to open cookies.json");
    let buffer = std::io::BufReader::new(cookie);

    let lark = lark::Session::new(&app_id, &app_secret);
    let wechat = wechat::Session::load(buffer).expect("failed to load wechat session");

    assert!(LARK.set(lark).is_ok(), "lark session already initialized");
    assert!(WECHAT.set(wechat).is_ok(), "wechat session already initialized");
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

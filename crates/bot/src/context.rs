//! 全局上下文

use std::env;
use std::sync::OnceLock;

// 全局上下文, 包含飞书, 微信和 LLM 的会话
static LARK: OnceLock<lark::Session> = OnceLock::new();
static WECHAT: OnceLock<wechat::Session> = OnceLock::new();
static LLM: OnceLock<crate::llm::Session> = OnceLock::new();

/// 初始化全局上下文, 并尝试恢复上次的会话状态
pub fn init() {
    // 初始化飞书会话
    let app_id = env::var("LARK_APP_ID").expect("LARK_APP_ID not set");
    let app_secret = env::var("LARK_APP_SECRET").expect("LARK_APP_SECRET not set");
    let lark = lark::Session::new(&app_id, &app_secret);
    assert!(LARK.set(lark).is_ok(), "lark already initialized");

    // 初始化微信会话
    let cookie = std::fs::File::open("cookies.json").expect("failed to open cookies.json");
    let buffer = std::io::BufReader::new(cookie);
    let wechat = wechat::Session::load(buffer).expect("failed to load wechat session");
    // 临时设置 token, 方便测试
    // let wechat_token = env::var("WECHAT_TOKEN").unwrap();
    // wechat.set_token(&wechat_token);
    assert!(WECHAT.set(wechat).is_ok(), "wechat already initialized");

    // 初始化 LLM 会话
    let llm_base_url = env::var("LLM_BASE_URL").expect("LLM_BASE_URL not set");
    let llm_api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY not set");
    let llm_model = env::var("LLM_MODEL").expect("LLM_MODEL not set");
    let llm = crate::llm::Session::new(&llm_base_url, &llm_api_key, &llm_model);
    assert!(LLM.set(llm).is_ok(), "llm already initialized");
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

/// 获取 LLM 会话
// TODO: 接入 AI 总结流程后移除 allow
#[allow(dead_code)]
pub fn llm() -> &'static crate::llm::Session {
    LLM.get()
        .expect("context not initialized, call context::init first")
}

/// 保存会话状态
pub fn save() {
    let cookie = std::fs::File::create("cookies.json").expect("failed to create cookies.json");
    let mut buffer = std::io::BufWriter::new(cookie);
    wechat()
        .save(&mut buffer)
        .expect("failed to save wechat session");
}

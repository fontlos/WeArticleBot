mod event;

use lark::WebSocketClient;

use std::env;
use std::sync::OnceLock;

static LARK: OnceLock<lark::Session> = OnceLock::new();
pub fn lark() -> &'static lark::Session {
    LARK.get().unwrap()
}

static WECHAT: OnceLock<wechat::Session> = OnceLock::new();
pub fn wechat() -> &'static wechat::Session {
    WECHAT.get().unwrap()
}

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
    // 初始化日志
    init_log();

    // 加载环境变量
    dotenvy::dotenv().ok();
    let app_id = env::var("APP_ID").unwrap();
    let app_secret = env::var("APP_SECRET").unwrap();

    let cookie = std::fs::File::open("cookies.json").unwrap();
    let buffer = std::io::BufReader::new(cookie);

    // 初始化 Lark Session, 供事件处理函数使用
    LARK.set(lark::Session::new(&app_id, &app_secret)).unwrap();

    // 初始化 WeChat Session, 供事件处理函数使用
    WECHAT.set(wechat::Session::load(buffer).unwrap()).unwrap();

    // 连接 WebSocket, 接收事件
    let mut websocket = WebSocketClient::connect(&app_id, &app_secret)
        .await
        .expect("Failed to initialize Lark bot");

    // 接收事件并处理
    while let Some(event) = websocket.recv().await {
        tokio::spawn(async move {
            event::handle(event).await;
        });
    }

    // let cookie = std::fs::File::open("cookie.json").unwrap();
    // let mut buffer = std::io::BufWriter::new(cookie);
    // wechat().save(&mut buffer).unwrap();
}

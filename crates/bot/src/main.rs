mod command;
mod context;
mod handler;
mod logs;

#[tokio::main]
async fn main() {
    // 加载 .env 文件
    dotenvy::dotenv().ok();
    // 初始化日志
    logs::init();
    // 初始化全局上下文 (Lark 和 WeChat 会话)
    context::init();

    // 基于会话凭据建立飞书长连接, 运行事件循环直到收到 Ctrl+C
    let websocket = context::lark()
        .connect_ws()
        .await
        .expect("Failed to initialize Lark bot");
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    websocket
        .run(ctrl_c, |event| handler::handle(event))
        .await
        .expect("websocket run failed");

    let cookie = std::fs::File::create("cookies.json").unwrap();
    let mut buffer = std::io::BufWriter::new(cookie);
    context::wechat().save(&mut buffer).unwrap();
}

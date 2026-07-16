mod command;
mod context;
mod handler;
mod logs;

#[tokio::main]
async fn main() {
    // 初始化日志
    logs::init();

    // 同步初始化全局上下文(env / cookie / 会话)
    context::init();

    // 建立飞书长连接, 运行事件循环直到收到 Ctrl+C
    let websocket = context::connect().await;
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

mod command;
mod context;
mod handler;
mod llm;
mod logs;

use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    // 加载 .env 文件
    dotenvy::dotenv().ok();
    // 初始化日志
    logs::init();
    // 初始化全局上下文 (Lark, WeChat 和 LLM 会话)
    context::init();

    // 停机信号
    let shutdown = CancellationToken::new();
    let signal = shutdown.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        signal.cancel();
    });

    // 建立长连接并运行事件循环
    context::lark()
        .run_ws(shutdown, |event| handler::handle(event))
        .await
        .expect("websocket run failed");

    // 保存会话状态
    context::save();
}

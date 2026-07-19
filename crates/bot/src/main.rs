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

    // 建立飞书长连接
    let websocket = context::lark()
        .connect_ws()
        .await
        .expect("Failed to initialize Lark bot");
    // 停机信号
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    // 运行事件循环
    websocket
        .run(ctrl_c, |event| handler::handle(event))
        .await
        .expect("websocket run failed");

    // 保存会话状态
    context::save();
}

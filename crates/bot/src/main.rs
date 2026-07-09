mod context;
mod handler;
mod logs;

#[tokio::main]
async fn main() {
    // 初始化日志
    logs::init();

    // 初始化全局上下文并建立飞书长连接
    let mut websocket = context::init().await;

    loop {
        tokio::select! {
            Some(event) = websocket.recv() => {
                tokio::spawn(async move {
                    handler::handle(event).await;
                });
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Received Ctrl+C");
                websocket.stop_graceful().await;
                break;
            }
        }
    }

    println!("WebSocket client stopped");

    let cookie = std::fs::File::create("cookies.json").unwrap();
    let mut buffer = std::io::BufWriter::new(cookie);
    context::wechat().save(&mut buffer).unwrap();
}

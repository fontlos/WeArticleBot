use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async_with_config as ws_connect;
use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};

use std::time::Duration;

use crate::error::Result;

use super::proto::Frame;
use super::config::{WsConfig, config};

pub struct WebSocketClient {
    event_receiver: mpsc::UnboundedReceiver<Bytes>,
}

impl WebSocketClient {
    pub async fn connect(app_id: &str, app_secret: &str) -> Result<Self> {
        let config = config(app_id, app_secret).await?;


        let (event_tx, event_rx) = mpsc::unbounded_channel::<Bytes>();
        // 启动 WebSocket 连接任务
        tokio::spawn(async move {
            if let Err(e) = run_websocket_loop(config, event_tx).await {
                eprintln!("WebSocket 循环错误: {}", e);
            }
        });

        Ok(Self {
            event_receiver: event_rx,
        })
    }
    /// 获取事件接收器
    pub async fn recv(&mut self) -> Option<Bytes> {
        self.event_receiver.recv().await
    }
}

async fn run_websocket_loop(
    config: WsConfig,
    event_tx: mpsc::UnboundedSender<Bytes>,
) -> Result<()> {
    let ws_config = WebSocketConfig::default()
        .max_message_size(Some(10 * 1024 * 1024))
        .max_frame_size(Some(10 * 1024 * 1024));

    let (ws_stream, _) = ws_connect(&config.url, Some(ws_config), false).await?;
    let (mut write, mut read) = ws_stream.split();

    // 任务通道
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Message>();

    // 心跳
    let ping = Duration::from_secs(config.config.ping as u64);
    let mut interval = tokio::time::interval(ping);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let _ = write.send(Message::Ping(vec![].into())).await;
                }
                Some(msg) = cmd_rx.recv() => {
                    let _ = write.send(msg).await;
                }
            }
        }
    });

    // 接收循环
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Binary(data) => {
                if let Ok(frame) = Frame::decode(data) {
                    if frame.method == 1 {
                        // 数据帧
                        if let Some(payload) = &frame.payload {
                            // 记录处理开始时间
                            let start = std::time::Instant::now();

                            // 异步事件循环
                            let event = Bytes::from(payload.to_vec());
                            let _ = event_tx.send(event);

                            // 计算处理时间
                            let elapsed = start.elapsed().as_millis();

                            // 复制一份 frame，只修改 payload
                            let mut response_frame = frame.clone();
                            response_frame.payload = Some(
                                serde_json::json!({
                                    "code": 200,
                                    "headers": {
                                        "biz_rt": elapsed.to_string()
                                    },
                                    "data": []
                                })
                                .to_string()
                                .into_bytes()
                            );

                            // 发送响应
                            let response_msg = Message::Binary(response_frame.encode_to_vec().into());
                            let _ = cmd_tx.send(response_msg);
                        }
                    }
                }
            }
            Message::Pong(_) => {
                println!("收到 Pong 响应");
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}

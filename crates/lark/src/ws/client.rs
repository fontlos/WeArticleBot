use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error};
use prost::Message as ProstMessage;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async_with_config as ws_connect;
use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};

use std::time::Duration;

use crate::error::Result;

use super::config::config;
use super::proto::Frame;

pub struct WebSocketClient {
    event_receiver: mpsc::UnboundedReceiver<Bytes>,
}

impl WebSocketClient {
    pub async fn connect(app_id: &str, app_secret: &str) -> Result<Self> {
        let config = config(app_id, app_secret).await?;
        // 建立 WebSocket 连接, 防止过大的消息导致的攻击
        let ws_config = WebSocketConfig::default()
            .max_message_size(Some(10 * 1024 * 1024))
            .max_frame_size(Some(10 * 1024 * 1024));
        // Websocket 的状态响应基本没用, 丢弃
        let (ws_stream, _) = ws_connect(&config.url, Some(ws_config), false).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        // 事件通道, 用于向外面发送响应事件, 让外部处理事件 JSON
        let (event_tx, event_rx) = mpsc::unbounded_channel::<Bytes>();
        // 响应事件通道, 用于发送响应帧, 通知服务器事件已收到
        let (resp_tx, mut resp_rx) = mpsc::unbounded_channel::<Message>();

        // Websocket 发送循环: 心跳 / 响应 Event
        let ping_interval = Duration::from_secs(config.config.ping as u64);
        let mut interval = tokio::time::interval(ping_interval);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let _ = ws_write.send(Message::Ping(Bytes::new())).await;
                    }
                    Some(msg) = resp_rx.recv() => {
                        let _ = ws_write.send(msg).await;
                    }
                }
            }
        });

        // Websocket 接收循环
        tokio::spawn(async move {
            while let Some(msg) = ws_read.next().await {
                let msg = match msg {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Websocket receive error: {}", e);
                        break;
                    }
                };
                match msg {
                    Message::Binary(data) => {
                        if let Ok(mut frame) = Frame::decode(data) {
                            // 1 是数据帧, 其他帧暂时不管
                            if frame.method != 1 {
                                debug!("Unknown frame frame, ignoring: \n{:?}", frame);
                                continue;
                            }
                            if let Some(payload) = frame.payload.take() {
                                // 异步事件循环, 发送处理事件
                                let event = Bytes::from(payload);
                                let _ = event_tx.send(event);
                                // 发送响应
                                frame.response(200);
                                let msg = Message::Binary(frame.encode_to_vec().into());
                                let _ = resp_tx.send(msg);
                            }
                        }
                    }
                    // 这里就是对 Ping 帧的回复, Ping 帧为空, 这里也为空
                    Message::Pong(_) => {
                        debug!("Websocket Pong");
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
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

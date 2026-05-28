use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error};
use prost::Message as ProstMessage;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async_with_config as ws_connect;
use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};

use std::time::Duration;

use crate::error::Result;

use super::config::config;
use super::proto::Frame;

pub struct WebSocketClient {
    event_rx: mpsc::UnboundedReceiver<Bytes>,
    shutdown_tx: watch::Sender<bool>,
    send_handle: Option<JoinHandle<()>>,
    recv_handle: Option<JoinHandle<()>>,
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
        // 停机信号通道
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        // 事件通道, 用于向外面发送响应事件, 让外部处理事件 JSON
        let (event_tx, event_rx) = mpsc::unbounded_channel::<Bytes>();
        // 响应事件通道, 用于发送响应帧, 通知服务器事件已收到
        let (resp_tx, mut resp_rx) = mpsc::unbounded_channel::<Message>();

        // Websocket 发送循环: 心跳 / 响应 Event / 关闭连接
        let ping_interval = Duration::from_secs(config.config.ping as u64);
        let mut interval = tokio::time::interval(ping_interval);
        let send_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 发送心跳帧
                    _ = interval.tick() => {
                        let _ = ws_write.send(Message::Ping(Bytes::new())).await;
                    }
                    // 发送响应帧
                    Some(msg) = resp_rx.recv() => {
                        let _ = ws_write.send(msg).await;
                    }
                    // 发送关闭帧
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            debug!("Sending Close frame to WebSocket server");
                            // 发送 Close 帧, 通过服务器回显 Close 帧将让下面的异步线程自然关闭
                            let msg = Message::Close(None);
                            let _ = ws_write.send(msg).await;
                            // 关闭连接
                            let _ = ws_write.close().await;
                            debug!("WebSocket send loop exited");
                            break;
                        }
                    }
                }
            }
        });

        // Websocket 接收循环
        let recv_handle = tokio::spawn(async move {
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
                        debug!("Websocket Pong frame received");
                    }
                    Message::Close(_) => {
                        debug!("WebSocket Close frame received");
                        break;
                    }
                    _ => {}
                }
            }
            debug!("WebSocket receive loop exited");
        });

        Ok(Self {
            event_rx,
            shutdown_tx,
            send_handle: Some(send_handle),
            recv_handle: Some(recv_handle),
        })
    }

    /// 获取事件接收器
    ///
    /// # Examples
    ///
    /// ```
    /// # use lark::ws::client::WebSocketClient;
    /// # async fn example() {
    /// # let client = WebSocketClient::connect("app_id", "app_secret").await.unwrap();
    /// while let Some(event) = websocket.recv().await {
    ///     tokio::spawn(async move {
    ///         handle(event).await;
    ///     });
    /// }
    /// # }
    /// ```
    pub async fn recv(&mut self) -> Option<Bytes> {
        self.event_rx.recv().await
    }

    pub async fn stop_graceful(&mut self) {
        // 发送关闭信号给后台任务
        let _ = self.shutdown_tx.send(true);
        // 关闭事件通道, 让上层循环退出
        self.event_rx.close();

        let shutdown = async {
            if let Some(handle) = self.send_handle.take() {
                let _ = handle.await;
            }
            if let Some(handle) = self.recv_handle.take() {
                let _ = handle.await;
            }
        };

        // 三秒后如果还没成功, 则强制终止异步线程, 防止 Close 帧丢失导致的僵尸线程
        match tokio::time::timeout(Duration::from_secs(3), shutdown).await {
            Ok(_) => {},
            Err(_) => {
                if let Some(handle) = self.send_handle.take() {
                    handle.abort();
                }
                if let Some(handle) = self.recv_handle.take() {
                    handle.abort();
                }
                error!("Timeout! WebSocket client aborted");
            }
        }

        debug!("WebSocket client stopped");
    }
}

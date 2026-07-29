use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, warn};
use prost::Message as ProstMessage;
use tokio::sync::mpsc;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;
use tokio_tungstenite::connect_async_with_config as ws_connect;
use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};

use std::future::Future;
use std::time::Duration;

use crate::error::Result;

use super::config::{ws_endpoint, WsEndpoint};
use super::proto::Frame;

/// 停机时等待 in-flight 处理任务的超时时间
const HANDLER_DRAIN_TIMEOUT: Duration = Duration::from_secs(3);
/// 事件通道容量: 满时 recv 循环暂停, 形成背压而非丢弃
const EVENT_CHANNEL_CAPACITY: usize = 1024;

/// 事件循环退出原因
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// 收到停机信号
    Shutdown,
    /// 连接断开/出错
    ConnectionLost,
}

pub struct WebSocketClient {
    event_rx: mpsc::Receiver<Bytes>,
    shutdown: CancellationToken,
    send_handle: Option<JoinHandle<()>>,
    recv_handle: Option<JoinHandle<()>>,
}

impl WebSocketClient {
    pub async fn connect(app_id: &str, app_secret: &str) -> Result<Self> {
        let endpoint = Self::get_endpoint(app_id, app_secret).await?;
        Self::connect_with_endpoint(endpoint).await
    }

    pub async fn get_endpoint(app_id: &str, app_secret: &str) -> Result<WsEndpoint> {
        ws_endpoint(app_id, app_secret).await
    }

    pub async fn connect_with_endpoint(endpoint: WsEndpoint) -> Result<Self> {
        // 建立 WebSocket 连接, 防止过大的消息导致的攻击
        let ws_config = WebSocketConfig::default()
            .max_message_size(Some(10 * 1024 * 1024))
            .max_frame_size(Some(10 * 1024 * 1024));
        // Websocket 的状态响应基本没用, 丢弃
        let (ws_stream, _) = ws_connect(&endpoint.url, Some(ws_config), false).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();
        // 内部停机信号
        // 停机流程:
        // 1. stop_graceful() cancel 内部 token, 关闭事件通道, 获取 send_handle, recv_handle 并挂起
        // 2. send_handle 通过 ws_write 发送 Close 帧, 关闭 ws 连接并退出
        // 3. recv_handle 通过 ws_read 接收 Close 帧并退出
        // 4. 等待 send_handle, recv_handle 退出, 超时则强制终止
        // 5. 等待后台任务退出, 超时则强制终止
        let shutdown = CancellationToken::new();
        let send_shutdown = shutdown.clone();

        // 事件通道, 用于向外面发送响应事件, 让外部处理事件 JSON
        // 有界: 满时 send().await 挂起 recv 循环, 停止读 socket 形成天然背压
        let (event_tx, event_rx) = mpsc::channel(EVENT_CHANNEL_CAPACITY);
        // 响应事件通道, 用于发送响应帧, 通知服务器事件已收到
        // 响应帧是取走了 payload 的原始帧, 足够小不会形成压力, 在内部是用无界通道即可
        let (resp_tx, mut resp_rx) = mpsc::unbounded_channel::<Message>();

        // Websocket 发送循环: 心跳 / 响应 Event / 关闭连接
        // 心跳间隔, 默认 90, 防御服务端返回 0 导致 interval(0) panic
        let ping_interval = Duration::from_secs(endpoint.config.ping.max(1) as u64);
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
                    _ = send_shutdown.cancelled() => {
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
                                // 通道满时挂起等待, 形成背压
                                let event = Bytes::from(payload);
                                let _ = event_tx.send(event).await;
                                // 发送响应(ack): 收帧即回 200, 与 handler 处理脱钩
                                // 配合分发器 event_id 去重保证不重复处理
                                // TODO: 如果以后需要"处理成功才 ack", 需把响应移到处理完成后
                                frame.response(200);
                                let msg = Message::Binary(frame.encode_to_vec().into());
                                let _ = resp_tx.send(msg);
                            }
                        }
                    }
                    // 服务器主动发来的 Ping, 需要回 Pong 保持连接健康
                    // 目前从未观察到飞书主动发起 Ping, 这里仅作防御
                    Message::Ping(payload) => {
                        let _ = resp_tx.send(Message::Pong(payload));
                    }
                    // 这里就是对 Ping 帧的回复, Ping 帧为空, 这里也为空
                    Message::Pong(_) => {
                        debug!("Websocket Pong frame received");
                    }
                    // 配合上面发送的 Close 帧, 收到回显 Close 帧自然关闭
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
            shutdown,
            send_handle: Some(send_handle),
            recv_handle: Some(recv_handle),
        })
    }

    /// 获取事件接收器
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lark::WebSocketClient;
    /// # async fn example() {
    /// # let mut client = WebSocketClient::connect("app_id", "app_secret").await.unwrap();
    /// while let Some(event) = client.recv().await {
    ///     tokio::spawn(async move {
    ///         // 分发事件给 handler
    ///     });
    /// }
    /// # }
    /// ```
    pub async fn recv(&mut self) -> Option<Bytes> {
        self.event_rx.recv().await
    }

    pub async fn stop_graceful(&mut self) {
        // 发送内部停机信号
        self.shutdown.cancel();
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
        match tokio::time::timeout(HANDLER_DRAIN_TIMEOUT, shutdown).await {
            Ok(_) => {}
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

    /// 运行事件循环, 直到收到关闭信号
    ///
    /// 整合 recv() 和 stop_graceful() 的调用, 返回退出原因
    pub async fn run<F, Fut, S>(mut self, shutdown: S, handler: &F) -> StopReason
    where
        S: Future<Output = ()> + Send,
        F: Fn(Bytes) -> Fut + Send + Sync + ?Sized,
        Fut: Future<Output = ()> + Send + 'static,
    {
        tokio::pin!(shutdown);
        let mut tasks = JoinSet::new();
        loop {
            tokio::select! {
                event = self.recv() => match event {
                    Some(event) => {
                        // 每事件一个任务, 由 JoinSet 统一管理
                        // TODO: 高并发存在资源耗尽压力, 可在 spawn 前用 Semaphore 限流
                        tasks.spawn(handler(event));
                    }
                    None => {
                        // 事件通道已关闭, 来源可能是:
                        // 1. 连接断开/出错: recv 任务退出, sender 被 drop, 应重连
                        // 2. 主动停机: 外部调用 stop_graceful 关闭了通道, 应视为 Shutdown
                        // TODO: 当前无法区分, 一律按 ConnectionLost 处理
                        // 目前 stop_graceful 仅由本函数调用, 无实际影响
                        debug!("Event channel closed, connection lost");
                        self.stop_graceful().await;
                        return StopReason::ConnectionLost;
                    }
                },
                _ = &mut shutdown => {
                    debug!("Shutdown signal received");
                    // 停止接收新事件并关闭连接
                    self.stop_graceful().await;
                    // 等待 in-flight 处理完成, 避免打断正在回复的消息
                    drain_tasks(&mut tasks).await;
                    return StopReason::Shutdown;
                }
            }
        }
    }
}

/// 等待所有 in-flight 处理任务完成, 超时则终止剩余任务
async fn drain_tasks(tasks: &mut JoinSet<()>) {
    let deadline = tokio::time::sleep(HANDLER_DRAIN_TIMEOUT);
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            joined = tasks.join_next() => {
                if joined.is_none() {
                    return; // 全部完成
                }
            }
            _ = &mut deadline => {
                warn!("Timeout draining in-flight handlers, aborting remaining");
                tasks.abort_all();
                return;
            }
        }
    }
}

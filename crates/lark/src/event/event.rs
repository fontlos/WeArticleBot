//! 飞书 WS 事件信封解析

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;

use crate::error::Result;

// WebSocket 事件信封
#[derive(Debug, Deserialize)]
pub struct EventEnvelope {
    // 这基本就是版本号, '2.0', 没什么用
    schema: String,
    header: EventHeader,
    // 装箱简化生命周期
    event: Box<RawValue>,
}

/// 事件头
#[derive(Debug, Deserialize)]
struct EventHeader {
    /// 事件 ID, 每个事件唯一, 可以用来去重
    event_id: String,
    event_type: String,
    create_time: String,
    tenant_key: String,
    app_id: String,
    // 似乎是空字段
    token: String,
}

impl EventEnvelope {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let envelope: EventEnvelope = serde_json::from_slice(bytes)?;
        Ok(envelope)
    }

    /// 版本号, 似乎总是'2.0', 没什么用
    #[inline]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// 事件 ID, 每个事件唯一, 可以用来去重
    #[inline]
    pub fn event_id(&self) -> &str {
        &self.header.event_id
    }

    /// 原始事件类型
    #[inline]
    pub fn event_type(&self) -> &str {
        &self.header.event_type
    }

    /// 事件时间戳, 毫秒字符串
    #[inline]
    pub fn timestamp(&self) -> &str {
        &self.header.create_time
    }

    /// 租户 ID, 可以用来区分不同的企业
    #[inline]
    pub fn tenant_key(&self) -> &str {
        &self.header.tenant_key
    }

    /// 不知道有什么用
    #[inline]
    pub fn app_id(&self) -> &str {
        &self.header.app_id
    }

    /// 似乎总是空的
    #[inline]
    pub fn token(&self) -> &str {
        &self.header.token
    }

    pub fn parse_event<E: DeserializeOwned>(&self) -> Result<E> {
        let event = serde_json::from_str(self.event.get())?;
        Ok(event)
    }
}

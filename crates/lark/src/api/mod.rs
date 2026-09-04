pub mod auth;
pub mod docs;
pub mod im;

mod data;

use crate::session::{Core, Session};

/// 授权 API 分组标记
pub struct Auth;
/// 云文档 API 分组标记
pub struct Docs;
/// 消息 API 分组标记
pub struct Im;

impl Session<Core> {
    /// 切换授权 API 分组实例
    pub fn auth(&self) -> &Session<Auth> {
        self.api()
    }

    /// 切换云文档 API 分组实例
    pub fn docs(&self) -> &Session<Docs> {
        self.api()
    }

    /// 切换消息 API 分组实例
    pub fn im(&self) -> &Session<Im> {
        self.api()
    }
}

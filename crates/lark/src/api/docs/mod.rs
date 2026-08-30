//! 云文档 API

pub mod bitable;
pub mod drive;
pub mod permission;

use crate::session::Session;

use super::Docs;

/// 多维表格 API 分组标记
pub struct Bitable;
/// 云空间 API 分组标记
pub struct Drive;
/// 权限 API 分组标记
pub struct Permission;

impl Session<Docs> {
    /// 切换多维表格 API 分组实例
    pub fn bitable(&self) -> &Session<Bitable> {
        self.api()
    }

    /// 切换云空间 API 分组实例
    pub fn drive(&self) -> &Session<Drive> {
        self.api()
    }

    /// 切换权限 API 分组实例
    pub fn permission(&self) -> &Session<Permission> {
        self.api()
    }
}

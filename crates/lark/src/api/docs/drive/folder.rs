//! 云文档 - 云空间 - 文件夹 API

use serde::Deserialize;

use crate::api::data::Res;
use crate::session::Session;

use super::super::Drive;

/// 文件夹元信息
#[derive(Debug, Deserialize)]
pub struct RootFolderMeta {
    pub token: String,
    pub id: String,
    pub user_id: String,
}

impl Session<Drive> {
    /// 获取根文件夹元信息
    ///
    /// 权限要求 (任选其一)
    /// - `drive:drive`
    /// - `drive:drive.metadata:readonly`
    pub async fn get_root_folder(&self) -> crate::Result<RootFolderMeta> {
        let url = "https://open.feishu.cn/open-apis/drive/explorer/v2/root_folder/meta";
        let req = self.client.get(url);
        let bytes = self.request(req).await?;
        let res: RootFolderMeta = Res::parse(&bytes)?;
        Ok(res)
    }
}

/// 文件列表
#[derive(Debug, Deserialize)]
pub struct FileList {
    pub files: Vec<FileMeta>,
    pub next_page_token: Option<String>,
    pub has_more: bool,
}

/// 文件元信息
#[derive(Debug, Deserialize)]
pub struct FileMeta {
    pub name: String,
    pub parent_token: String,
    pub token: String,
    pub r#type: String,
    pub created_time: String,
    pub modified_time: String,
    pub owner_id: String,
    pub url: String,
}

impl Session<Drive> {
    /// 获取根文件夹元信息
    ///
    /// 权限要求 (任选其一)
    /// - `drive:drive`
    /// - `drive:drive:readonly`
    /// - `space:document:retrieve`
    pub async fn get_file_list(&self) -> crate::Result<FileList> {
        let url = "https://open.feishu.cn/open-apis/drive/v1/files";
        let req = self.client.get(url);
        let bytes = self.request(req).await?;
        let res: FileList = Res::parse(&bytes)?;
        Ok(res)
    }
}

/// 文件夹 trait
pub trait Folder {
    fn token(&self) -> &str;
    // 根文件夹一定是文件夹, 其他文件元信息需要检查 type
    fn is_folder(&self) -> bool {
        true
    }
}

impl Folder for RootFolderMeta {
    fn token(&self) -> &str {
        &self.token
    }
}

impl Folder for FileMeta {
    fn token(&self) -> &str {
        &self.token
    }
    fn is_folder(&self) -> bool {
        self.r#type == "folder"
    }
}

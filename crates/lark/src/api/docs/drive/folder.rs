//! 云文档 - 云空间 - 文件夹 API

use serde::Deserialize;

use crate::api::response::Res;
use crate::session::Session;

/// 文件夹元信息
#[derive(Debug, Deserialize)]
pub struct FolderMeta {
    pub token: String,
    pub id: String,
    pub user_id: String,
}

impl Session {
    /// 获取根文件夹元信息
    ///
    /// 权限要求 (任选其一)
    /// - `drive:drive`
    /// - `drive:drive.metadata:readonly`
    pub async fn get_root_folder(&self) -> crate::Result<FolderMeta> {
        let url = "https://open.feishu.cn/open-apis/drive/explorer/v2/root_folder/meta";
        let req = self.client.get(url);
        let bytes = self.request(req).await?;
        let res: FolderMeta = Res::parse(&bytes)?;
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

impl Session {
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

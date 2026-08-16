//! 云文档 - 云空间 - 文件夹 API

use serde::Deserialize;

use crate::session::Session;
use crate::api::response::Res;

#[derive(Debug, Deserialize)]
pub struct FolderMeta {
    pub token: String,
    pub id: String,
    pub user_id: String,
}

impl Session {
    /// 获取根文件夹元信息
    pub async fn get_root_folder(&self) -> crate::Result<FolderMeta> {
        let url = "https://open.feishu.cn/open-apis/drive/explorer/v2/root_folder/meta";
        let req = self.client.get(url);
        let bytes = self.request(req).await?;
        let res: FolderMeta = Res::parse(&bytes)?;
        Ok(res)
    }
}

//! 云文档 - 云空间 - 文件 API

use serde::Deserialize;

use crate::api::Res;
use crate::api::docs::drive::folder::{FileMeta, Folder};
use crate::error::Error;
use crate::session::Session;

use super::super::Drive;

/// 移动文件夹属于异步任务, 使用 ID 查询执行情况
#[derive(Debug, Deserialize)]
struct Task {
    task_id: String,
}

#[derive(Debug, Deserialize)]
struct Status {
    // success
    status: String,
}

impl Session<Drive> {
    /// 检查异步任务状态
    pub async fn check_task(&self, task_id: &str) -> crate::Result<bool> {
        let url = "https://open.feishu.cn/open-apis/drive/v1/files/task_check";
        let query = [("task_id", task_id)];
        let req = self.client.get(url).query(&query);
        let bytes = self.request(req).await?;
        let res: Status = Res::parse(&bytes)?;
        Ok(res.status == "success")
    }

    /// 移动文件到指定文件夹
    ///
    /// 需要 RootFolderMeta 或者 type 为 folder 的 FileMeta
    pub async fn move_file<F>(&self, from: &FileMeta, to: &F) -> crate::Result<String>
    where
        F: Folder,
    {
        if !to.is_folder() {
            return Err(Error::Custom(
                "`to` must be a folder".to_string(),
            ));
        }
        let url = format!(
            "https://open.feishu.cn/open-apis/drive/v1/files/{}/move",
            from.token
        );
        let body = serde_json::json!({
            "type": from.r#type,
            "folder_token": to.token(),
        });
        let req = self.client.post(&url).json(&body);
        let bytes = self.request(req).await?;
        let res: Task = Res::parse(&bytes)?;
        Ok(res.task_id)
    }

    /// 删除文件
    pub async fn delete_file(&self, file: &FileMeta) -> crate::Result<String> {
        let url = format!(
            "https://open.feishu.cn/open-apis/drive/v1/files/{}",
            file.token
        );
        let query = [("type", file.r#type.as_str())];
        let req = self.client.delete(&url).query(&query);
        let bytes = self.request(req).await?;
        let res: Task = Res::parse(&bytes)?;
        Ok(res.task_id)
    }
}

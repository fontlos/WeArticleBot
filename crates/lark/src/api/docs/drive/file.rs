//! 云文档 - 云空间 - 文件 API

use serde::Deserialize;

use crate::api::Res;
use crate::api::docs::drive::folder::FileMeta;
use crate::session::Session;

use super::super::Drive;

/// 移动文件夹属于异步任务, 使用 ID 查询执行情况
#[derive(Debug, Deserialize)]
struct Task {
    pub task_id: String,
}

impl Session<Drive> {
    pub async fn move_file(&self, from: &FileMeta, to: &str) -> crate::Result<String> {
        let url = format!(
            "https://open.feishu.cn/open-apis/drive/v1/files/{}/move",
            from.token
        );
        let body = serde_json::json!({
            "type": from.r#type,
            "folder_token": to,
        });
        let req = self.client.post(&url).json(&body);
        let bytes = self.request(req).await?;
        let res: Task = Res::parse(&bytes)?;
        Ok(res.task_id)
    }
}

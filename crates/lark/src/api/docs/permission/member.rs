//! 云文档 - 权限 - 成员 API

use serde::Deserialize;

use crate::Session;

use crate::api::Res;
use crate::api::docs::drive::folder::FileMeta;

use super::super::Permission;

#[derive(Debug, Deserialize)]
pub struct Member {
    pub member_type: String,
    pub member_id: String,
    pub perm: String,
    pub perm_type: String,
    pub r#type: String,
}

impl Session<Permission> {
    /// 获取成员信息
    /// TODO: 暂时只给 OpenID 全部权限
    pub async fn add_member(&self, file: &FileMeta, open_id: &str) -> crate::Result<Member> {
        let url = format!(
            "https://open.feishu.cn/open-apis/drive/v1/permissions/{}/members",
            file.token
        );
        let query = [("type", file.r#type.as_str())];
        let json = serde_json::json!({
            "member_type": "openid",
            "member_id": open_id,
            "perm": "full_access",
        });
        let req = self.client.post(&url).query(&query).json(&json);
        let bytes = self.request(req).await?;
        let res: Member = Res::parse(&bytes)?;
        Ok(res)
    }
}

//! 云文档 - 权限 - 成员 API

use serde::Deserialize;

use crate::Session;

use crate::api::data::Res;

use super::super::Permission;

#[derive(Debug, Deserialize)]
struct MemberWrap {
    member: Member,
}

#[derive(Debug, Deserialize)]
pub struct Member {
    pub member_type: String,
    pub member_id: String,
    pub perm: String,
    pub perm_type: String,
    pub r#type: Option<String>,
}

impl Session<Permission> {
    /// 添加成员权限
    /// TODO: 暂时只给 OpenID 全部权限
    /// doc_type: 文档类型, 如 "bitable"/"docx"/"sheet"
    pub async fn add_member(
        &self,
        token: &str,
        doc_type: &str,
        open_id: &str,
    ) -> crate::Result<Member> {
        let url = format!(
            "https://open.feishu.cn/open-apis/drive/v1/permissions/{}/members",
            token
        );
        let query = [("type", doc_type)];
        let json = serde_json::json!({
            "member_type": "openid",
            "member_id": open_id,
            "perm": "full_access",
        });
        let req = self.client.post(&url).query(&query).json(&json);
        let bytes = self.request(req).await?;
        let res: MemberWrap = Res::parse(&bytes)?;
        Ok(res.member)
    }
}

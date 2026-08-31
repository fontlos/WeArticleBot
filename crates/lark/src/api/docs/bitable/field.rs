//! 云文档 - 多维表格 - 字段 API

use serde::Deserialize;
use serde_json::Value;

use crate::Session;

use crate::api::Res;

use super::super::Bitable;

#[derive(Deserialize)]
struct FieldWrap {
    field: FieldRes,
}

/// 新增字段响应
#[derive(Debug, Deserialize)]
pub struct FieldRes {
    pub field_id: String,
    pub field_name: String,
}

impl Session<Bitable> {
    /// 新增字段
    pub async fn create_field(
        &self,
        app_token: &str,
        table_id: &str,
        field: &Value,
    ) -> crate::Result<FieldRes> {
        let url = format!(
            "https://open.feishu.cn/open-apis/bitable/v1/apps/{}/tables/{}/fields",
            app_token, table_id
        );
        let req = self.client.post(&url).json(&field);
        let bytes = self.request(req).await?;

        let res: FieldWrap = Res::parse(&bytes)?;
        Ok(res.field)
    }
}

//! 云文档 - 多维表格 - 记录 API

use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

use crate::Session;

use crate::api::Res;

use super::super::Bitable;

/// 记录
#[derive(Debug, Deserialize)]
pub struct Record {
    pub record_id: String,
    pub fields: Value,
}

#[derive(Deserialize)]
struct RecordWrap {
    record: Record,
}

impl Session<Bitable> {
    /// 新增单条记录
    pub async fn create_record(
        &self,
        app_token: &str,
        table_id: &str,
        fields: &Value,
    ) -> crate::Result<Record> {
        let url = format!(
            "https://open.feishu.cn/open-apis/bitable/v1/apps/{}/tables/{}/records",
            app_token, table_id
        );
        let body = json!({ "fields": fields });
        let req = self.client.post(&url).json(&body);
        let bytes = self.request(req).await?;

        let res: RecordWrap = Res::parse(&bytes)?;
        Ok(res.record)
    }
}

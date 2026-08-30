//! 云文档 - 多维表格 - 数据表 API

use serde::Deserialize;
use serde_json::Value;

use crate::Session;

use crate::api::Res;
use crate::api::docs::drive::folder::FileMeta;

use super::super::Bitable;

#[derive(Debug, Deserialize)]
pub struct TableRes {
    pub table_id: String,
    pub default_view_id: Option<String>,
    pub field_id_list: Option<Vec<String>>,
}

impl Session<Bitable> {
    /// 创建数据表
    /// TODO: 数据表结构过于复杂先手动构造 JSON 传入
    pub async fn create_table(&self, bitable: &FileMeta, table: &Value) -> crate::Result<TableRes>{
        let url = format!("https://open.feishu.cn/open-apis/bitable/v1/apps/{}/tables", bitable.token);
        let req = self.client.post(url).json(&table);
        let bytes = self.request(req).await?;
        let res: TableRes = Res::parse(&bytes)?;
        Ok(res)
    }
}
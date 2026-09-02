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

#[derive(Deserialize)]
struct RecordsWrap {
    records: Vec<Record>,
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

    /// 批量新增记录 (单次最多 500 条)
    ///
    /// records: 每条为字段名 -> 值 的映射(与 create_record 的 fields 一致),
    /// 内部会包一层 \"fields\" 后再提交
    pub async fn batch_create_records(
        &self,
        app_token: &str,
        table_id: &str,
        records: &[Value],
    ) -> crate::Result<Vec<Record>> {
        let url = format!(
            "https://open.feishu.cn/open-apis/bitable/v1/apps/{}/tables/{}/records/batch_create",
            app_token, table_id
        );
        let records: Vec<Value> = records
            .iter()
            .map(|fields| json!({ "fields": fields }))
            .collect();
        let body = json!({ "records": records });
        let req = self.client.post(&url).json(&body);
        let bytes = self.request(req).await?;

        let res: RecordsWrap = Res::parse(&bytes)?;
        Ok(res.records)
    }
}

/// 记录列表(查询结果)
#[derive(Debug, Deserialize)]
pub struct RecordList {
    #[serde(default)]
    pub items: Vec<Record>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub page_token: Option<String>,
    #[serde(default)]
    pub total: i32,
}

impl Session<Bitable> {
    /// 列出记录(GET, 不支持条件筛选/排序; 如需筛选请用 search_records)
    pub async fn list_records(
        &self,
        app_token: &str,
        table_id: &str,
        page_size: usize,
    ) -> crate::Result<RecordList> {
        let url = format!(
            "https://open.feishu.cn/open-apis/bitable/v1/apps/{}/tables/{}/records",
            app_token, table_id
        );
        let req = self
            .client
            .get(&url)
            .query(&[("page_size", page_size.to_string())]);
        let bytes = self.request(req).await?;

        let res: RecordList = Res::parse(&bytes)?;
        Ok(res)
    }

    /// 搜索记录(POST /records/search, 支持条件筛选 + 排序)
    ///
    /// filter: 形如 {"conjunction":"and","conditions":[{"field_name":"处理状态","operator":"is","value":["待总结"]}]}
    /// sort:   形如 [{"field_name":"发布时间","desc":true}]
    pub async fn search_records(
        &self,
        app_token: &str,
        table_id: &str,
        filter: Option<&Value>,
        sort: Option<&Value>,
        page_size: usize,
    ) -> crate::Result<RecordList> {
        let url = format!(
            "https://open.feishu.cn/open-apis/bitable/v1/apps/{}/tables/{}/records/search",
            app_token, table_id
        );
        let mut body = serde_json::Map::new();
        if let Some(f) = filter {
            body.insert("filter".into(), f.clone());
        }
        if let Some(s) = sort {
            body.insert("sort".into(), s.clone());
        }
        let req = self
            .client
            .post(&url)
            .query(&[("page_size", page_size.to_string())])
            .json(&body);
        let bytes = self.request(req).await?;

        let res: RecordList = Res::parse(&bytes)?;
        Ok(res)
    }

    /// 更新记录(部分字段)
    pub async fn update_record(
        &self,
        app_token: &str,
        table_id: &str,
        record_id: &str,
        fields: &Value,
    ) -> crate::Result<Record> {
        let url = format!(
            "https://open.feishu.cn/open-apis/bitable/v1/apps/{}/tables/{}/records/{}",
            app_token, table_id, record_id
        );
        let body = json!({ "fields": fields });
        let req = self.client.put(&url).json(&body);
        let bytes = self.request(req).await?;

        let res: RecordWrap = Res::parse(&bytes)?;
        Ok(res.record)
    }
}

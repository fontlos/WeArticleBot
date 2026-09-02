//! bot 运行时状态(暂存)
//!
//! 多维表格创建后把 app_token 与各表 id 落到本地文件, 供后续命令复用。

use serde::{Deserialize, Serialize};

const BITABLE_FILE: &str = "bitable.json";

/// 多维表格定位信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitableState {
    pub app_token: String,
    pub accounts_table_id: String,
    pub articles_table_id: String,
    pub summaries_table_id: String,
}

impl BitableState {
    pub fn save(&self) {
        let bytes = serde_json::to_vec(self).expect("serialize bitable state");
        std::fs::write(BITABLE_FILE, bytes).expect("write bitable state");
    }

    pub fn load() -> Option<Self> {
        let bytes = std::fs::read(BITABLE_FILE).ok()?;
        serde_json::from_slice(&bytes).ok()
    }
}

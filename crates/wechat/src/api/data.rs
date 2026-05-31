use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct Res<T> {
    pub base_resp: BaseRes,
    #[serde(flatten)]
    pub data: T,
}

impl<'de, T: Deserialize<'de>> Res<T> {
    pub fn parse(bytes: &'de [u8]) -> Result<T> {
        let res: Res<T> = serde_json::from_slice(bytes)?;
        if res.base_resp.ret != 0 {
            return Err(Error::Custom(format!(
                "API error: {}",
                res.base_resp.err_msg
            )));
        }
        Ok(res.data)
    }
}

#[derive(Debug, Deserialize)]
pub struct BaseRes {
    pub ret: i32,
    pub err_msg: String,
}

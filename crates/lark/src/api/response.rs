use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct Response {
    pub code: i32,
    pub msg: String,
}

impl Response {
    /// 检查响应
    pub fn check(bytes: &[u8]) -> Result<()> {
        let res: Response = serde_json::from_slice(bytes)?;
        if res.code != 0 {
            return Err(Error::Custom(format!(
                "Bad response: code {}, message: {}",
                res.code, res.msg
            )));
        }
        Ok(())
    }

    /// 解析响应
    pub fn parse<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T> {
        Self::check(bytes)?;
        #[derive(Deserialize)]
        struct Data<T> {
            data: Option<T>,
        }
        let res: Data<T> = serde_json::from_slice(bytes)?;
        match res.data {
            Some(data) => Ok(data),
            None => Err(Error::Custom("Bad response: no `data` field".into())),
        }
    }
}

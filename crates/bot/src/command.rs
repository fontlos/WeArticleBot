//! 字符串命令解析

use clap::Parser;

#[derive(Debug, Parser)]
pub enum Cli {
    /// 获取微信登录二维码
    Login,
    /// 获取用户信息
    Info,
    /// 搜索公众号
    Search {
        /// 搜索关键词
        keyword: String,
    },
}

pub fn parse_cli(text: &str) -> Result<Cli, clap::Error> {
    let args = text.split_whitespace();
    // 添加程序名作为第一个参数
    let args = std::iter::once("bot").chain(args);
    Cli::try_parse_from(args)
}

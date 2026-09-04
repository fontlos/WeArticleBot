//! 字符串命令解析

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 获取微信登录二维码
    Login,
    /// 初始化多维表格与权限
    Init,
    /// 搜索公众号
    Search {
        /// 搜索关键词
        keyword: String,
    },
    /// 添加订阅公众号
    Add { index: u8 },
    /// 同步公众号的文章到表格
    Sync,
    /// 总结待总结文章
    Summary,
    /// 查询各组件状态
    Query(Query),
}

#[derive(Debug, Parser)]
pub struct Query {
    #[clap(subcommand)]
    pub command: QuerySub,
}

#[derive(Debug, Subcommand)]
pub enum QuerySub {
    /// 查询飞书用户信息
    Lark,
    /// 查询微信用户信息
    Wechat,
}

pub fn parse_cli(text: &str) -> Result<Cli, clap::Error> {
    let args = text.split_whitespace();
    // 添加程序名作为第一个参数
    let args = std::iter::once("bot").chain(args);
    Cli::try_parse_from(args)
}

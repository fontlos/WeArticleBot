//! 字符串命令解析

/// 命令类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Help,
    Info,
    Login,
    Search,
}

/// 参数规格
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgSpec {
    pub name: &'static str,
    pub required: bool,
}

impl ArgSpec {
    /// 必填参数
    pub const fn required(name: &'static str) -> Self {
        Self {
            name,
            required: true,
        }
    }

    /// 可选参数
    pub const fn optional(name: &'static str) -> Self {
        Self {
            name,
            required: false,
        }
    }
}

/// 命令元数据
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgSpec],
    pub kind: Kind,
}

impl CommandSpec {
    /// 生成用法字符串
    pub fn usage(&self) -> String {
        let mut usage = String::with_capacity(16 + self.args.len() * 8);
        usage.push_str(self.name);
        for arg in self.args {
            if arg.required {
                usage.push_str(" <");
            } else {
                usage.push_str(" [");
            }
            usage.push_str(arg.name);
            usage.push(if arg.required { '>' } else { ']' });
        }
        usage
    }

    pub fn min_args(&self) -> usize {
        self.args.iter().filter(|a| a.required).count()
    }

    pub fn max_args(&self) -> usize {
        self.args.len()
    }
}

/// 命令表
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "help",
        description: "显示帮助信息",
        args: &[ArgSpec::optional("命令")],
        kind: Kind::Help,
    },
    CommandSpec {
        name: "info",
        description: "获取微信个人信息",
        args: &[],
        kind: Kind::Info,
    },
    CommandSpec {
        name: "login",
        description: "获取微信登录二维码",
        args: &[],
        kind: Kind::Login,
    },
    CommandSpec {
        name: "search",
        description: "搜索公众号",
        args: &[ArgSpec::required("关键词")],
        kind: Kind::Search,
    },
];

/// 解析结果
#[derive(Debug, PartialEq)]
pub struct Parsed {
    pub kind: Kind,
    pub args: Vec<String>,
}

/// 解析错误
#[derive(Debug, PartialEq)]
pub enum Error {
    /// 未知命令
    Unknown(String),
    /// 无效参数
    InvalidArgs {
        spec: &'static CommandSpec,
        reason: InvalidArgs,
    },
}

/// 参数错误的具体原因
#[derive(Debug, PartialEq)]
pub enum InvalidArgs {
    MissingArg(&'static str),
    TooManyArgs,
}

/// 解析入口
pub fn parse(text: &str) -> Result<Parsed, Error> {
    let mut parts = text.split_whitespace();
    let Some(name) = parts.next() else {
        return Ok(Parsed {
            kind: Kind::Help,
            args: vec![],
        });
    };
    let args: Vec<String> = parts.map(str::to_owned).collect();

    // 按名字查表, 命令类型直接取 spec.kind
    let Some(spec) = COMMANDS.iter().find(|c| c.name == name) else {
        return Err(Error::Unknown(name.to_owned()));
    };

    if args.len() < spec.min_args() {
        let missing = spec
            .args
            .iter()
            .find(|a| a.required)
            .expect("min_args > 0 implies a required arg exists")
            .name;
        return Err(Error::InvalidArgs {
            spec,
            reason: InvalidArgs::MissingArg(missing),
        });
    }
    if args.len() > spec.max_args() {
        return Err(Error::InvalidArgs {
            spec,
            reason: InvalidArgs::TooManyArgs,
        });
    }

    Ok(Parsed {
        kind: spec.kind,
        args,
    })
}

/// 生成全部命令的帮助文本
pub fn general_help() -> String {
    let mut text = String::from("可用命令:\n");
    for spec in COMMANDS {
        text.push_str(&format!("- {}  {}\n", spec.usage(), spec.description));
    }
    text
}

/// 未知命令提示文本
pub fn unknown_text(name: &str) -> String {
    format!("未知命令: {name}\n输入 help 查看可用命令")
}

/// 无效参数提示文本
pub fn invalid_text(spec: &CommandSpec, reason: &InvalidArgs) -> String {
    match reason {
        InvalidArgs::MissingArg(arg) => {
            format!("用法: {}（缺少必填参数 <{arg}>）", spec.usage())
        }
        InvalidArgs::TooManyArgs => format!("用法: {}（参数过多）", spec.usage()),
    }
}

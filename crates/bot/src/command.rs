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
    pub const fn required(name: &'static str) -> Self {
        Self {
            name,
            required: true,
        }
    }

    pub const fn optional(name: &'static str) -> Self {
        Self {
            name,
            required: false,
        }
    }
}

/// 命令元数据(帮助与校验的唯一数据源)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgSpec],
    pub kind: Kind,
}

impl CommandSpec {
    /// 生成用法字符串, 例如 "search <关键词>" / "help [命令]"
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

/// 命令表: 编译期常量, 名字/描述/参数规格/类型唯一出处
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

/// 解析结果: 命令类型 + 已校验的参数列表
#[derive(Debug, PartialEq)]
pub struct Parsed {
    pub kind: Kind,
    pub args: Vec<String>,
}

/// 解析错误
#[derive(Debug, PartialEq)]
pub enum Error {
    /// 未注册的命令
    Unknown(String),
    /// 参数个数不满足命令定义
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

    // 按名字查表, 命令类型直接取 spec.kind, 不手写字符串映射
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

/// 生成单个命令的帮助文本
pub fn command_help(name: &str) -> Option<String> {
    COMMANDS.iter().find(|c| c.name == name).map(|spec| {
        format!(
            "{}: {}\n用法: {}",
            spec.name,
            spec.description,
            spec.usage()
        )
    })
}

/// 未知命令提示文本
pub fn unknown_text(name: &str) -> String {
    format!("未知命令: {name}\n输入 help 查看可用命令")
}

/// 参数错误提示文本
pub fn invalid_text(spec: &CommandSpec, reason: &InvalidArgs) -> String {
    match reason {
        InvalidArgs::MissingArg(arg) => {
            format!("用法: {}（缺少必填参数 <{arg}>）", spec.usage())
        }
        InvalidArgs::TooManyArgs => format!("用法: {}（参数过多）", spec.usage()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(name: &str) -> &'static CommandSpec {
        COMMANDS.iter().find(|c| c.name == name).unwrap()
    }

    #[test]
    fn parse_basic() {
        assert_eq!(
            parse("  search   关键词  "),
            Ok(Parsed {
                kind: Kind::Search,
                args: vec!["关键词".into()]
            })
        );
        assert_eq!(
            parse("info"),
            Ok(Parsed {
                kind: Kind::Info,
                args: vec![]
            })
        );
        assert_eq!(
            parse("login"),
            Ok(Parsed {
                kind: Kind::Login,
                args: vec![]
            })
        );
    }

    #[test]
    fn parse_empty_is_general_help() {
        assert_eq!(
            parse("   "),
            Ok(Parsed {
                kind: Kind::Help,
                args: vec![]
            })
        );
    }

    #[test]
    fn parse_help_with_target() {
        assert_eq!(
            parse("help login"),
            Ok(Parsed {
                kind: Kind::Help,
                args: vec!["login".into()]
            })
        );
    }

    #[test]
    fn parse_unknown_command() {
        assert_eq!(parse("nope"), Err(Error::Unknown("nope".into())));
    }

    #[test]
    fn parse_missing_required_arg() {
        assert_eq!(
            parse("search"),
            Err(Error::InvalidArgs {
                spec: spec("search"),
                reason: InvalidArgs::MissingArg("关键词"),
            })
        );
    }

    #[test]
    fn parse_too_many_args() {
        assert_eq!(
            parse("search a b"),
            Err(Error::InvalidArgs {
                spec: spec("search"),
                reason: InvalidArgs::TooManyArgs,
            })
        );
        assert_eq!(
            parse("info extra"),
            Err(Error::InvalidArgs {
                spec: spec("info"),
                reason: InvalidArgs::TooManyArgs,
            })
        );
    }

    #[test]
    fn parse_maps_spec_to_kind() {
        // 反向映射: 每个命令名解析出的类型必须与表内 kind 一致
        for s in COMMANDS {
            match parse(s.name) {
                Ok(parsed) => assert_eq!(parsed.kind, s.kind, "{}", s.name),
                Err(Error::Unknown(_)) => panic!("spec 未映射: {}", s.name),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn usage_is_derived() {
        assert_eq!(spec("search").usage(), "search <关键词>");
        assert_eq!(spec("help").usage(), "help [命令]");
        assert_eq!(spec("info").usage(), "info");
    }

    #[test]
    fn help_lists_all_commands() {
        let text = general_help();
        for s in COMMANDS {
            assert!(text.contains(s.name), "help 缺少命令: {}", s.name);
            assert!(text.contains(s.description), "help 缺少描述: {}", s.name);
        }
    }

    #[test]
    fn command_help_known_and_unknown() {
        assert_eq!(
            command_help("search").unwrap(),
            "search: 搜索公众号\n用法: search <关键词>"
        );
        assert_eq!(command_help("nope"), None);
    }

    #[test]
    fn error_texts() {
        assert_eq!(unknown_text("foo"), "未知命令: foo\n输入 help 查看可用命令");
        assert_eq!(
            invalid_text(spec("search"), &InvalidArgs::MissingArg("关键词")),
            "用法: search <关键词>（缺少必填参数 <关键词>）"
        );
    }
}

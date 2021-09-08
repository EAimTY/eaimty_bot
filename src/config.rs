use crate::error::ConfigError;
use getopts::Options;

pub struct Config {
    pub token: String,
    pub proxy: Option<String>,
    pub webhook_port: u16,
}

impl Config {
    pub fn parse(args: Vec<String>) -> Result<Self, ConfigError> {
        // 定义命令行参数
        let mut opts = Options::new();
        opts.optopt(
            "t",
            "token",
            "(required) set Telegram Bot HTTP API token",
            "TOKEN",
        );
        opts.optopt(
            "p",
            "proxy",
            "set proxy (supported: http, https, socks5)",
            "PROXY",
        );
        opts.optopt(
            "w",
            "webhook-port",
            "set webhook port (1 ~ 65535) and run bot in webhook mode",
            "WEBHOOK_PORT",
        );
        opts.optflag("h", "help", "print this help menu");
        // 定义帮助信息
        let usage = opts.usage(&format!("Usage: {} [options]", args[0]));
        // 尝试解析传入的命令行参数
        let matches = opts
            .parse(&args[1..])
            .or_else(|_| return Err(ConfigError::ParseError(usage.clone())))?;
        // 若有未定义的参数报错
        if !matches.free.is_empty() {
            return Err(ConfigError::UnexpectedFragment(usage.clone()));
        };
        // 若传入 -h 参数，返回帮助信息
        if matches.opt_present("h") {
            return Err(ConfigError::Help(usage.clone()));
        }
        // 处理传入的参数
        let token = if let Some(token) = matches.opt_str("t") {
            token
        } else {
            return Err(ConfigError::ParseError(usage.clone()));
        };
        let proxy = matches.opt_str("p");
        let webhook_port = matches
            .opt_str("w")
            .unwrap_or(String::from("0"))
            .parse::<u16>()
            .unwrap_or(0);
        Ok(Self {
            token,
            proxy,
            webhook_port,
        })
    }
}

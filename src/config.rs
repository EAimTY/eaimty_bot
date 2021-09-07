use getopts::Options;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ConfigErrorType {
    ParseError,
    UnexpectedFragment,
    Help,
}

#[derive(Debug)]
pub struct ConfigError {
    error_type: ConfigErrorType,
    usage: String,
}

impl ConfigError {
    pub fn new(error_type: ConfigErrorType, usage: &str) -> Self {
        Self {
            error_type,
            usage: usage.to_string(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error_type {
            ConfigErrorType::ParseError => write!(f, "Failed to parse arguments\n{}", self.usage),
            ConfigErrorType::UnexpectedFragment => write!(f, "Unexpected fragment\n{}", self.usage),
            ConfigErrorType::Help => write!(f, "{}", self.usage),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

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
            .or_else(|_| return Err(ConfigError::new(ConfigErrorType::ParseError, &usage)))?;
        // 若有未定义的参数报错
        if !matches.free.is_empty() {
            return Err(ConfigError::new(
                ConfigErrorType::UnexpectedFragment,
                &usage,
            ));
        };
        // 若传入 -h 参数，返回帮助信息
        if matches.opt_present("h") {
            return Err(ConfigError::new(ConfigErrorType::Help, &usage));
        }
        // 处理传入的参数
        let token = if let Some(token) = matches.opt_str("t") {
            token
        } else {
            return Err(ConfigError::new(ConfigErrorType::ParseError, &usage));
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

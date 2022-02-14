use anyhow::{anyhow, bail, Result};
use getopts::Options;
use reqwest::Proxy;

pub struct Config {
    pub token: String,
    pub webhook_port: Option<u16>,
    pub proxy: Option<Proxy>,
}

pub struct ConfigBuilder<'a> {
    opts: Options,
    program: Option<&'a str>,
}

impl<'a> ConfigBuilder<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut opts = Options::new();

        opts.reqopt(
            "t",
            "token",
            "Set the Telegram Bot HTTP API token (required)",
            "TOKEN",
        );

        opts.optopt(
            "w",
            "webhook-port",
            "Run in webhook mode listening port (1 ~ 65535)",
            "WEBHOOK_PORT",
        );

        opts.optopt(
            "",
            "proxy",
            "Set proxy  (supported: http, https, socks5)",
            "PROXY",
        );

        opts.optflag("v", "version", "Print the version");
        opts.optflag("h", "help", "Print this help menu");

        Self {
            opts,
            program: None,
        }
    }

    pub fn get_usage(&self) -> String {
        self.opts.usage(&format!(
            "Usage: {} [options]",
            self.program.unwrap_or(env!("CARGO_PKG_NAME"))
        ))
    }

    pub fn parse(&mut self, args: &'a [String]) -> Result<Config> {
        self.program = Some(&args[0]);

        let matches = self
            .opts
            .parse(&args[1..])
            .map_err(|err| anyhow!("{err}\n\n{}", self.get_usage()))?;

        if !matches.free.is_empty() {
            bail!(
                "Unexpected argument: {}\n\n{}",
                matches.free.join(", "),
                self.get_usage()
            );
        }

        if matches.opt_present("v") {
            bail!("{}", env!("CARGO_PKG_VERSION"));
        }

        if matches.opt_present("h") {
            bail!("{}", self.get_usage());
        }

        let token = unsafe { matches.opt_str("t").unwrap_unchecked() };

        let webhook_port = if let Some(port) = matches.opt_str("w") {
            let port = port
                .parse()
                .map_err(|err| anyhow!("{err}\n\n{}", self.get_usage()))?;

            if port == 0 {
                bail!(
                    "Port 0 cannot be used as the webhook port\n\n{}",
                    self.get_usage()
                );
            }

            Some(port)
        } else {
            None
        };

        let proxy = if let Some(proxy) = matches.opt_str("proxy") {
            Some(Proxy::all(&proxy).map_err(|err| anyhow!("{err}\n\n{}", self.get_usage()))?)
        } else {
            None
        };

        Ok(Config {
            token,
            webhook_port,
            proxy,
        })
    }
}

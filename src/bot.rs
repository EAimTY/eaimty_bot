use crate::{Config, Database, Handler};
use anyhow::Result;
use reqwest::Client;
use tgbot::{longpoll::LongPoll, methods::GetMe, types::Me, webhook, Api};

pub async fn run(config: Config) -> Result<()> {
    let http_cli = {
        let mut builder = Client::builder();

        if let Some(proxy) = config.proxy {
            builder = builder.proxy(proxy);
        }

        builder.build()?
    };

    let api = Api::with_client(http_cli, config.token);
    let db = Database::new();

    let Me { username, .. } = api.execute(GetMe).await?;
    let username = format!("@{username}");

    if let Some(webhook_port) = config.webhook_port {
        webhook::run_server(
            ([0, 0, 0, 0], webhook_port),
            "/",
            Handler::new(api, db, username),
        )
        .await?;
    } else {
        LongPoll::new(api.clone(), Handler::new(api, db, username))
            .run()
            .await;
    }

    Ok(())
}

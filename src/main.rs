use crate::{config::Config, server::Server};
use std::env;

mod config;
mod context;
mod error;
mod handlers;
mod server;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::parse(args) {
        Ok(config) => config,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    Server::run(config)
        .await
        .unwrap_or_else(|err| println!("{}", err));
}

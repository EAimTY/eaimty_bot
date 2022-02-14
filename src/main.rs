pub use crate::{
    config::{Config, ConfigBuilder},
    database::Database,
    handler::{Context, Handler},
};
use std::{env, process};

mod bot;
mod config;
mod database;
mod handler;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let mut cfg_builder = ConfigBuilder::new();

    let cfg = match cfg_builder.parse(&args) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    match bot::run(cfg).await {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
}

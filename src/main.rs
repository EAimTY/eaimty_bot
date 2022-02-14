pub use crate::{
    config::{Config, ConfigBuilder},
    database::Database,
    handler::Handler,
};
use std::{env, process, time::Duration};

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

    let (db, gc) = Database::init(Duration::from_secs(3600), Duration::from_secs(3));
    tokio::spawn(gc);

    match bot::run(cfg, db).await {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
}

use crate::context::Context;
use carapax::{
    Api, Config, Dispatcher, webhook,
    longpoll::LongPoll,
    session::{backend::fs::FilesystemBackend, SessionCollector, SessionManager}
};
use clap::{App, Arg};
use std::time::Duration;
use tempfile::tempdir;
use tokio::spawn;

mod context;
mod error;
mod handlers;

async fn run(token: &str, proxy: &str, webhook: &str) {
    let mut config = Config::new(token);
    if !proxy.is_empty() {
        config = config.proxy(proxy).expect("Failed to set proxy");
    }
    let api = Api::new(config).expect("Failed to create API");
    let tmpdir = tempdir().expect("Failed to create temp directory");
    let backend = FilesystemBackend::new(tmpdir.path());
    let gc_period = Duration::from_secs(3);
    let session_lifetime = Duration::from_secs(86400);
    let mut collector = SessionCollector::new(backend.clone(), gc_period, session_lifetime);
    spawn(async move {
        collector.run().await
    });
    let mut dispatcher = Dispatcher::new(Context {
        api: api.clone(),
        session_manager: SessionManager::new(backend),
        tmpdir: tmpdir
    });
    dispatcher.add_handler(handlers::about::about_command_handler);
    dispatcher.add_handler(handlers::agree::agree_keyword_handler);
    dispatcher.add_handler(handlers::dart::dart_command_handler);
    dispatcher.add_handler(handlers::dart::dart_keyword_handler);
    dispatcher.add_handler(handlers::dice::dice_command_handler);
    dispatcher.add_handler(handlers::dice::dice_keyword_handler);
    dispatcher.add_handler(handlers::ocr::ocr_command_handler);
    dispatcher.add_handler(handlers::ocr::ocr_image_handler);
    dispatcher.add_handler(handlers::ocr::ocr_inlinekeyboard_handler);
    dispatcher.add_handler(handlers::othello::othello_command_handler);
    dispatcher.add_handler(handlers::othello::othello_inlinekeyboard_handler);
    dispatcher.add_handler(handlers::slot::slot_command_handler);
    dispatcher.add_handler(handlers::start::start_command_handler);
    dispatcher.add_handler(handlers::tictactoe::tictactoe_command_handler);
    dispatcher.add_handler(handlers::tictactoe::tictactoe_inlinekeyboard_handler);
    let webhook_port = webhook.parse::<u16>().unwrap_or(0);
    if webhook_port == 0 {
        println!("Running in longpoll mode");
        LongPoll::new(api, dispatcher).run().await;
    } else {
        println!("Running at port {} in webhook mode", webhook_port);
        webhook::run_server(([127, 0, 0, 1], webhook_port), "/", dispatcher).await.expect("Failed to run webhook server");
    }
}

#[tokio::main]
async fn main() {
    let matches = App::new("eaimty_bot")
        .about("A Telegram Bot")
        .arg(Arg::with_name("token")
            .short("t")
            .long("token")
            .value_name("TOKEN")
            .help("Sets HTTP API token")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("proxy")
            .short("p")
            .long("proxy")
            .value_name("PROXY")
            .help("Sets proxy (supported: http, https, socks5)")
            .takes_value(true))
        .arg(Arg::with_name("webhook")
            .short("w")
            .long("webhook")
            .value_name("PORT")
            .help("Runs in webhook mode. Sets port number as the argument value")
            .takes_value(true))
        .get_matches();
    run(matches.value_of("token").unwrap(), matches.value_of("proxy").unwrap_or(""), matches.value_of("webhook").unwrap_or("")).await;
}
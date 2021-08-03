use carapax::{
    Api, Config, Dispatcher, webhook,
    longpoll::LongPoll,
    session::{backend::fs::FilesystemBackend, SessionCollector, SessionManager}
};
use clap::{App, Arg};
use std::time::Duration;
use tempfile::{TempDir, tempdir};
use tokio::spawn;

mod commands;
mod keywords;

pub struct Context {
    api: Api,
    session_manager: SessionManager<FilesystemBackend>,
    tmpdir: TempDir
}

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
    dispatcher.add_handler(commands::dice::dice_command_handler);
    dispatcher.add_handler(commands::dart::dart_command_handler);
    dispatcher.add_handler(commands::ocr::ocr_command_handler);
    dispatcher.add_handler(commands::ocr::ocr_inlinekeyboard_handler);
    dispatcher.add_handler(commands::ocr::ocr_image_handler);
    dispatcher.add_handler(keywords::dice::dice_keyword_handler);
    dispatcher.add_handler(keywords::dart::dart_keyword_handler);
    dispatcher.add_handler(keywords::unanimity::unanimity_keyword_handler);
    let webhook_port = match webhook.parse::<u16>() {
        Ok(port) => port,
        Err(_) => 0
    };
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
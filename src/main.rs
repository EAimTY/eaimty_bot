use carapax::{
    Api, Config, Dispatcher,
    session::{backend::fs::FilesystemBackend, SessionCollector, SessionManager},
    longpoll::LongPoll
};
use tempfile::{tempdir, TempDir};
use std::time::Duration;
use clap::{Arg, App};

mod commands;
mod keywords;

pub struct Context {
    api: Api,
    session_manager: SessionManager<FilesystemBackend>,
    tmpdir: TempDir
}

async fn run(token: &str, proxy: &str) {
    let mut config = Config::new(token);
    if proxy != "" {
        config = config.proxy(proxy).expect("Failed to set proxy");
    }
    let api = Api::new(config).expect("Failed to create API");

    let tmpdir = tempdir().expect("Failed to create temp directory");
    let backend = FilesystemBackend::new(tmpdir.path());
    let gc_period = Duration::from_secs(3);
    let session_lifetime = Duration::from_secs(86400);
    let mut collector = SessionCollector::new(backend.clone(), gc_period, session_lifetime);
    tokio::spawn(
        async move {
            collector.run().await
        }
    );

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
    LongPoll::new(api, dispatcher).run().await;
}

#[tokio::main]
async fn main() {
    let matches = App::new("eaimty_bot")
        .about("A Telegram Bot")
        .arg(Arg::with_name("token")
            .short("t")
            .long("token")
            .value_name("TOKEN")
            .help("Sets token")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("proxy")
            .short("p")
            .long("proxy")
            .value_name("PROXY")
            .help("Sets proxy (supported: http, https, socks5)")
            .takes_value(true))
        .get_matches();
    run(matches.value_of("token").unwrap(), matches.value_of("proxy").unwrap_or("")).await;
}

use crate::context::Context;
use carapax::{
    longpoll::LongPoll,
    session::{backend::fs::FilesystemBackend, SessionCollector, SessionManager},
    webhook, Api, Config, Dispatcher,
};
use getopts::Options;
use std::{env, time::Duration};
use tempfile::tempdir;
use tokio::spawn;

mod context;
mod error;
mod handlers;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
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
    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(_) => {
            print_help(&program, opts);
            return;
        }
    };
    if !matches.free.is_empty() {
        print_help(&program, opts);
        return;
    };
    if matches.opt_present("h") {
        print_help(&program, opts);
        return;
    }
    let token = matches.opt_str("t");
    let proxy = matches.opt_str("p");
    let webhook_port = matches.opt_str("w");
    match token {
        Some(token) => run(token, proxy, webhook_port).await,
        None => {
            print_help(&program, opts);
            return;
        }
    }
}

fn print_help(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

async fn run(token: String, proxy: Option<String>, webhook_port: Option<String>) {
    let mut config = Config::new(token);
    if let Some(proxy) = proxy {
        config = config.proxy(proxy).expect("Failed to set proxy");
    }
    let api = Api::new(config).expect("Failed to create API");
    let tmpdir = tempdir().expect("Failed to create temp directory");
    let backend = FilesystemBackend::new(tmpdir.path());
    let gc_period = Duration::from_secs(3);
    let session_lifetime = Duration::from_secs(86400);
    let mut collector = SessionCollector::new(backend.clone(), gc_period, session_lifetime);
    spawn(async move { collector.run().await });
    let mut dispatcher = Dispatcher::new(Context {
        api: api.clone(),
        session_manager: SessionManager::new(backend),
        tmpdir: tmpdir,
    });
    dispatcher.add_handler(handlers::access::group_message_filter);
    dispatcher.add_handler(handlers::about::about_command_handler);
    dispatcher.add_handler(handlers::agree::agree_keyword_handler);
    dispatcher.add_handler(handlers::dart::dart_command_handler);
    dispatcher.add_handler(handlers::dart::dart_keyword_handler);
    dispatcher.add_handler(handlers::dice::dice_command_handler);
    dispatcher.add_handler(handlers::dice::dice_keyword_handler);
    dispatcher.add_handler(handlers::minesweeper::minesweeper_command_handler);
    dispatcher.add_handler(handlers::minesweeper::minesweeper_inlinekeyboard_handler);
    dispatcher.add_handler(handlers::ocr::ocr_command_handler);
    dispatcher.add_handler(handlers::ocr::ocr_image_handler);
    dispatcher.add_handler(handlers::ocr::ocr_inlinekeyboard_handler);
    dispatcher.add_handler(handlers::othello::othello_command_handler);
    dispatcher.add_handler(handlers::othello::othello_inlinekeyboard_handler);
    dispatcher.add_handler(handlers::slot::slot_command_handler);
    dispatcher.add_handler(handlers::start::start_command_handler);
    dispatcher.add_handler(handlers::tictactoe::tictactoe_command_handler);
    dispatcher.add_handler(handlers::tictactoe::tictactoe_inlinekeyboard_handler);
    let webhook_port = webhook_port
        .unwrap_or(String::from("0"))
        .parse::<u16>()
        .unwrap_or(0);
    if webhook_port == 0 {
        println!("Running in longpoll mode");
        LongPoll::new(api, dispatcher).run().await;
    } else {
        println!("Running at port {} in webhook mode", webhook_port);
        webhook::run_server(([127, 0, 0, 1], webhook_port), "/", dispatcher)
            .await
            .expect("Failed to run webhook server");
    }
}

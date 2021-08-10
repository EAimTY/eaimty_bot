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
    opts.optopt("t", "token", "set token", "TOKEN");
    opts.optopt("p", "proxy", "set proxy", "PROXY");
    opts.optopt("w", "webhook-port", "set webhook port", "WEBHOOK_PORT");
    opts.optflag("v", "version", "print version");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            panic!("{}", e)
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let token = matches.opt_str("t");
    if let None = token {
        print_usage(&program, opts);
        return;
    }
    let proxy = matches.opt_str("p");
    let webhook_port = matches.opt_str("w");
    if !matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    };
    run(token, proxy, webhook_port).await;
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

async fn run(token: Option<String>, proxy: Option<String>, webhook_port: Option<String>) {
    let mut config = Config::new(token.unwrap());
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
    match webhook_port {
        Some(port) => {
            let port = port.parse::<u16>().unwrap_or(0);
            println!("Running at port {} in webhook mode", port);
            webhook::run_server(([127, 0, 0, 1], port), "/", dispatcher)
                .await
                .expect("Failed to run webhook server");
        }
        None => {
            println!("Running in longpoll mode");
            LongPoll::new(api, dispatcher).run().await;
        }
    }
}

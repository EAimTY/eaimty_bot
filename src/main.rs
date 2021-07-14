use carapax::{longpoll::LongPoll, Api, Config, Dispatcher};
use clap::{Arg, App};

mod commands;
mod keywords;

async fn run(token: &str, proxy: &str) {
    let mut config = Config::new(token);
    if proxy != "" {
        config = config.proxy(proxy).expect("Failed to set proxy");
    }
    let api = Api::new(config).expect("Failed to create API");
    let mut dispatcher = Dispatcher::new(api.clone());
    dispatcher.add_handler(commands::dice_command::handle_dice);
    dispatcher.add_handler(commands::dart_command::handle_dart);
    dispatcher.add_handler(keywords::dice_keyword::dice_handler);
    dispatcher.add_handler(keywords::dart_keyword::dart_handler);
    dispatcher.add_handler(keywords::unanimity_keyword::unanimity_handler);
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

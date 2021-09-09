use crate::{config::Config, context::Context, error::ServerError, handlers};
use carapax::{
    longpoll::LongPoll,
    session::{backend::fs::FilesystemBackend, SessionCollector, SessionManager},
    webhook, Api, Config as ApiConfig, Dispatcher,
};
use std::time::Duration;

pub struct Server;

impl Server {
    pub async fn run(config: Config) -> Result<(), ServerError> {
        // API 设置
        let mut api_config = ApiConfig::new(config.token);
        if let Some(proxy) = config.proxy {
            api_config = api_config
                .proxy(proxy)
                .or_else(|_| return Err(ServerError::ProxyError))?;
        }
        // 创建 API
        let api = Api::new(api_config).or_else(|_| return Err(ServerError::ApiError))?;
        // 创建临时文件目录
        let tmpdir = tempfile::tempdir().or_else(|_| return Err(ServerError::TmpdirError))?;
        // 创建 session 管理器与其垃圾回收线程
        let backend = FilesystemBackend::new(tmpdir.path());
        let gc_period = Duration::from_secs(3);
        let session_lifetime = Duration::from_secs(86400);
        let mut collector = SessionCollector::new(backend.clone(), gc_period, session_lifetime);
        tokio::spawn(async move { collector.run().await });
        // 创建 dispatcher
        let mut dispatcher = Dispatcher::new(Context::new(
            api.clone(),
            SessionManager::new(backend),
            tmpdir,
        )?);
        // 添加 handlers
        dispatcher.add_handler(handlers::access::set_bot_command);
        dispatcher.add_handler(handlers::access::group_message_filter);
        dispatcher.add_handler(handlers::about::about_command_handler);
        dispatcher.add_handler(handlers::agree::agree_command_handler);
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
        // 添加错误处理 handler
        dispatcher.set_error_handler(handlers::ErrorHandler);
        // 运行
        if config.webhook_port == 0 {
            println!("Running in longpoll mode");
            LongPoll::new(api, dispatcher).run().await;
        } else {
            println!("Running at port {} in webhook mode", config.webhook_port);
            webhook::run_server(([127, 0, 0, 1], config.webhook_port), "/", dispatcher)
                .await
                .or_else(|_| return Err(ServerError::WebhookServerError))?;
        }
        Ok(())
    }
}

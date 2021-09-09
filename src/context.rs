use crate::{error::ServerError, handlers};
use carapax::{
    session::{backend::fs::FilesystemBackend, SessionManager},
    types::BotCommand,
    Api,
};
use tempfile::TempDir;
use tokio::sync::RwLock;

// 定义 bot 命令列表
pub struct BotCommands {
    pub is_set: RwLock<bool>,
    pub command_list: Vec<BotCommand>,
}

// 定义用于创建 bot 命令列表的宏
#[macro_export]
macro_rules! bot_commands {
    ( $( $command:expr ),* ) => {
        {
            let mut command_list = Vec::new();
            $(
                let command = BotCommand::new($command.0, $command.1);
                match command {
                    Ok(command) => command_list.push(command),
                    Err(_) => return Err(ServerError::BotCommandListError),
                }
            )*
            Ok(command_list)
        }
    };
}

impl BotCommands {
    fn init() -> Result<Self, ServerError> {
        // 在此处设置 bot 命令列表
        let command_list = bot_commands!(
            ("agree", "没有，没有，没有，好，通过！"),
            ("dart", "掷一枚飞标（可通过汉字“飞标”触发）"),
            ("dice", "掷一枚骰子（可通过汉字“骰子”触发）"),
            ("minesweeper", "玩扫雷"),
            ("ocr", "识别图片中文字"),
            ("othello", "玩黑白棋"),
            ("slot", "转一次老虎机"),
            ("tictactoe", "玩 Tic-Tac-Toe"),
            ("about", "关于本 bot")
        )?;
        Ok(Self {
            is_set: RwLock::new(false),
            command_list,
        })
    }
}

pub struct Context {
    pub api: Api,
    pub session_manager: SessionManager<FilesystemBackend>,
    pub tmpdir: TempDir,
    pub bot_info: RwLock<Option<handlers::access::BotInfo>>,
    pub ocr_langs: handlers::ocr::OcrLangs,
    pub bot_commands: BotCommands,
}

impl Context {
    pub fn new(
        api: Api,
        session_manager: SessionManager<FilesystemBackend>,
        tmpdir: TempDir,
    ) -> Result<Self, ServerError> {
        Ok(Self {
            api,
            session_manager,
            tmpdir,
            bot_info: RwLock::new(None),
            ocr_langs: handlers::ocr::OcrLangs::init(),
            bot_commands: BotCommands::init()?,
        })
    }
}

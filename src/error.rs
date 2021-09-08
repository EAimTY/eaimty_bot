use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // 无法通过 Telegram Bot API 执行操作的错误
    #[error("failed to execute method: {0}")]
    ExecuteError(#[from] carapax::ExecuteError),
    // 无法获取 session 的错误
    #[error("failed to get session")]
    GetSessionError,
    // 无法从 session 读写数据的错误
    #[error("failed to read / write data from session")]
    SessionDataError,
    // 无法操作 IO 的错误
    #[error("failed to operate file: {0}")]
    IoError(#[from] std::io::Error),
    // 下载文件的错误
    #[error("failed to download file")]
    FileDownloadError,
    // Tesseract 初始化错误
    #[error("failed to initial Tesseract")]
    TessInitError,
    // 无法读取图片的错误
    #[error("failed to read image for Tesseract")]
    TessReadImageError,
}

// 配置参数错误
#[derive(Error, Debug)]
pub enum ConfigError {
    // 参数解析错误
    #[error("Failed to parse arguments\n{0}")]
    ParseError(String),
    // 未知参数的错误
    #[error("Unexpected fragment\n{0}")]
    UnexpectedFragment(String),
    // 帮助信息
    #[error("{0}")]
    Help(String),
}

// Telegram Bot API 通信错误
#[derive(Error, Debug)]
pub enum ServerError {
    // API 创建错误
    #[error("Failed to create API")]
    ApiError,
    // 代理设置错误
    #[error("Failed to set proxy")]
    ProxyError,
    // 临时文件目录创建错误
    #[error("Failed to create temp directory")]
    TmpdirError,
    // Webhook Server 运行错误
    #[error("Failed to run webhook server")]
    WebhookServerError,
}

// Tic-Tac-Toe 错误操作
#[derive(Error, Debug)]
pub enum TicTacToeOpError {
    // 在非空白处落子
    #[error("请在空白处落子")]
    CellNotEmpty,
    // 在非己方回合落子
    #[error("不是你的回合")]
    NotYourTurn,
}

// 黑白棋错误操作
#[derive(Error, Debug)]
pub enum OthelloOpError {
    // 在无法落子处落子
    #[error("无法在此落子")]
    CantPutHere,
    // 在非己方回合落子
    #[error("不是你的回合")]
    NotYourTurn,
}

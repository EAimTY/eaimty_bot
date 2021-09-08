use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // 无法通过 Telegram Bot API 执行操作的错误
    #[error("failed to execute method")]
    ExecuteError(#[from] carapax::ExecuteError),
    // 无法获取 session 的错误
    #[error("failed to get session")]
    GetSessionError,
    // 无法从 session 读写数据的错误
    #[error("failed to read / write data from session")]
    SessionDataError,
    // 无法操作 IO 的错误
    #[error("failed to operate file")]
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

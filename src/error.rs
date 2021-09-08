use carapax;
use std::io;
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
    IoError(#[from] io::Error),
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

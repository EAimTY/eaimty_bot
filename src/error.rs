use carapax::{DownloadFileError, ExecuteError, session::SessionError};
use std::{convert::Infallible, error::Error, fmt, io::Error as IOError};
use reqwest::Error as ReqwestError;
use tesseract::TesseractError;

#[derive(Debug)]
pub enum ErrorHandler {
    DownloadFileError(DownloadFileError),
    ExecuteError(ExecuteError),
    Infallible(Infallible),
    IOError(IOError),
    ReqwestError(ReqwestError),
    SessionError(SessionError),
    TesseractError(TesseractError)
}

impl From<DownloadFileError> for ErrorHandler {
    fn from(err: DownloadFileError) -> Self {
        ErrorHandler::DownloadFileError(err)
    }
}

impl From<ExecuteError> for ErrorHandler {
    fn from(err: ExecuteError) -> Self {
        ErrorHandler::ExecuteError(err)
    }
}

impl From<Infallible> for ErrorHandler {
    fn from(err: Infallible) -> Self {
        ErrorHandler::Infallible(err)
    }
}

impl From<IOError> for ErrorHandler {
    fn from(err: IOError) -> Self {
        ErrorHandler::IOError(err)
    }
}

impl From<ReqwestError> for ErrorHandler {
    fn from(err: ReqwestError) -> Self {
        ErrorHandler::ReqwestError(err)
    }
}

impl From<SessionError> for ErrorHandler {
    fn from(err: SessionError) -> Self {
        ErrorHandler::SessionError(err)
    }
}

impl From<TesseractError> for ErrorHandler {
    fn from(err: TesseractError) -> Self {
        ErrorHandler::TesseractError(err)
    }
}

impl fmt::Display for ErrorHandler {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorHandler::*;
        match self {
            DownloadFileError(err) => write!(out, "failed to download file: {}", err),
            ExecuteError(err) => write!(out, "failed to execute method: {}", err),
            Infallible(err) => write!(out, "infallible error: {}", err),
            IOError(err) => write!(out, "can not operate file: {}", err),
            ReqwestError(err) => write!(out, "failed to process request: {}", err),
            SessionError(err) => write!(out, "failed to operate session: {}", err),
            TesseractError(err) => write!(out, "failed to operate ocr: {}", err)
        }
    }
}

impl Error for ErrorHandler {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::ErrorHandler::*;
        Some(match self {
            DownloadFileError(err) => err,
            ExecuteError(err) => err,
            Infallible(err) => err,
            IOError(err) => err,
            ReqwestError(err) => err,
            SessionError(err) => err,
            TesseractError(err) => err
        })
    }
}
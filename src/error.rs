use carapax::{
    session::{SessionError, SessionIdError},
    DownloadFileError, ExecuteError,
};
use leptess::{leptonica::PixError, tesseract::TessInitError};
use reqwest::Error as ReqwestError;
use std::{convert::Infallible, error::Error, fmt, io::Error as IOError};

#[derive(Debug)]
pub enum ErrorHandler {
    DownloadFileError(DownloadFileError),
    ExecuteError(ExecuteError),
    Infallible(Infallible),
    IOError(IOError),
    PixError(PixError),
    ReqwestError(ReqwestError),
    SessionError(SessionError),
    SessionIdError(SessionIdError),
    TessInitError(TessInitError),
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

impl From<PixError> for ErrorHandler {
    fn from(err: PixError) -> Self {
        ErrorHandler::PixError(err)
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

impl From<SessionIdError> for ErrorHandler {
    fn from(err: SessionIdError) -> Self {
        ErrorHandler::SessionIdError(err)
    }
}

impl From<TessInitError> for ErrorHandler {
    fn from(err: TessInitError) -> Self {
        ErrorHandler::TessInitError(err)
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
            PixError(err) => write!(out, "can not read image file: {}", err),
            ReqwestError(err) => write!(out, "failed to process request: {}", err),
            SessionError(err) => write!(out, "failed to operate session: {}", err),
            SessionIdError(err) => write!(out, "failed to get session id: {}", err),
            TessInitError(err) => write!(out, "failed to initiate tesseract: {}", err),
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
            PixError(err) => err,
            ReqwestError(err) => err,
            SessionError(err) => err,
            SessionIdError(err) => err,
            TessInitError(err) => err,
        })
    }
}

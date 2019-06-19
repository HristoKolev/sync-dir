use std::convert::From;
use core::fmt;

use backtrace::Backtrace;

use self::CustomErrorKind::*;

pub enum CustomErrorKind {
    ErrorMessage(String),
    PanicErrorMessage(String),
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    PoisonedError(String),
    ParseIntError(std::num::ParseIntError),
    UrlParseError(url::ParseError),
    ReqwestError(reqwest::Error),
    SystemTimeError(std::time::SystemTimeError),

    #[allow(unused)]
    UserError(String),
    NotifyError(notify::Error),
    IgnoreError(ignore::Error),
}

#[derive(Debug)]
pub struct CustomError {
    pub kind: CustomErrorKind,
    pub backtrace: Backtrace,
}

impl fmt::Debug for CustomErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorMessage(err) => return err.fmt(f),
            PanicErrorMessage(err) => return err.fmt(f),
            IoError(err) => return err.fmt(f),
            JsonError(err) => return err.fmt(f),
            PoisonedError(err) => return err.fmt(f),
            ParseIntError(err) => return err.fmt(f),
            UrlParseError(err) => return err.fmt(f),
            ReqwestError(err) => return err.fmt(f),
            SystemTimeError(err) => return err.fmt(f),
            UserError(err) => return err.fmt(f),
            NotifyError(err) => return err.fmt(f),
            IgnoreError(err) => return err.fmt(f),
        };
    }
}

impl ToString for CustomErrorKind {
    fn to_string(&self) -> String {
        match self {
            ErrorMessage(err) => return err.to_string(),
            PanicErrorMessage(err) => return err.to_string(),
            IoError(err) => return err.to_string(),
            JsonError(err) => return err.to_string(),
            PoisonedError(err) => return err.to_string(),
            ParseIntError(err) => return err.to_string(),
            UrlParseError(err) => return err.to_string(),
            ReqwestError(err) => return err.to_string(),
            SystemTimeError(err) => return err.to_string(),
            UserError(err) => return err.to_string(),
            NotifyError(err) => return err.to_string(),
            IgnoreError(err) => return err.to_string(),
        }
    }
}

impl CustomError {
    pub fn from_message(message: &str) -> CustomError {
        CustomError {
            kind: ErrorMessage(message.to_string()),
            backtrace: Backtrace::new(),
        }
    }

    #[allow(unused)]
    pub fn user_error(message: &str) -> CustomError {
        CustomError {
            kind: UserError(message.to_string()),
            backtrace: Backtrace::new(),
        }
    }

    pub fn from_panic_message(message: &str, backtrace: Backtrace) -> CustomError {
        CustomError {
            kind: PanicErrorMessage(message.to_string()),
            backtrace,
        }
    }
}

impl From<std::io::Error> for CustomError {
    fn from(err: std::io::Error) -> Self {
        CustomError {
            kind: IoError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<serde_json::Error> for CustomError {
    fn from(err: serde_json::Error) -> Self {
        CustomError {
            kind: JsonError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<ignore::Error> for CustomError {
    fn from(err: ignore::Error) -> Self {
        CustomError {
            kind: IgnoreError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<notify::Error> for CustomError {
    fn from(err: notify::Error) -> Self {
        CustomError {
            kind: NotifyError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<std::num::ParseIntError> for CustomError {
    fn from(err: std::num::ParseIntError) -> Self {
        CustomError {
            kind: ParseIntError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<url::ParseError> for CustomError {
    fn from(err: url::ParseError) -> Self {
        CustomError {
            kind: UrlParseError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<reqwest::Error> for CustomError {
    fn from(err: reqwest::Error) -> Self {
        CustomError {
            kind: ReqwestError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<std::time::SystemTimeError> for CustomError {
    fn from(err: std::time::SystemTimeError) -> Self {
        CustomError {
            kind: SystemTimeError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, super::logging::FileAppenderState>>> for CustomError {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'_, super::logging::FileAppenderState>>) -> Self {
        CustomError {
            kind: PoisonedError(format!("{:#?}", err)),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, Vec<String>>>> for CustomError {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'_, Vec<String>>>) -> Self {
        CustomError {
            kind: PoisonedError(format!("{:#?}", err)),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, std::collections::HashMap<std::string::String, std::boxed::Box<(dyn std::ops::Fn() -> std::result::Result<(), crate::global::errors::CustomError> + std::marker::Send + std::marker::Sync + 'static)>>>>> for CustomError {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'_, std::collections::HashMap<std::string::String, std::boxed::Box<(dyn std::ops::Fn() -> std::result::Result<(), crate::global::errors::CustomError> + std::marker::Send + std::marker::Sync + 'static)>>>>) -> Self {
        CustomError {
            kind: PoisonedError(format!("{:#?}", err)),
            backtrace: Backtrace::new(),
        }
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, bool>>> for CustomError {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'_, bool>>) -> Self {
        CustomError {
            kind: PoisonedError(format!("{:#?}", err)),
            backtrace: Backtrace::new(),
        }
    }
}

pub type Result<T = ()> = std::result::Result<T, CustomError>;

pub trait ResultExtensionsReplaceError<R> {
    fn replace_error<ErrFunc>(self, err_func: ErrFunc) -> Result<R>
        where ErrFunc: FnOnce() -> CustomError;
}

impl<R, E> ResultExtensionsReplaceError<R> for std::result::Result<R, E> {
    fn replace_error<ErrFunc>(self, err_func: ErrFunc) -> Result<R>
        where ErrFunc: FnOnce() -> CustomError {
        match self {
            Ok(res) => Ok(res),
            Err(_) => Err(err_func())
        }
    }
}

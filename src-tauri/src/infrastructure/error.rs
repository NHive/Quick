use sea_orm::DbErr;
use serde::Serialize;
use serde_json::Error as JsonError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    IOError(#[from] io::Error),

    #[error(transparent)]
    DbError(#[from] DbErr),

    #[error(transparent)]
    JsonError(#[from] JsonError),

    #[error("数据验证错误: {0}")]
    ValidationError(String),

    #[error("资源未找到: {0}")]
    NotFoundError(String),

    #[error("未知错误: {0}")]
    UnknownError(String),
}

#[allow(dead_code)]
impl AppError {
    pub(crate) fn error_code(&self) -> i32 {
        match self {
            AppError::IOError(_) => 10001,
            AppError::DbError(_) => 10002,
            AppError::JsonError(_) => 10003,
            AppError::ValidationError(_) => 10004,
            AppError::NotFoundError(_) => 10005,

            AppError::UnknownError(_) => 19999,
        }
    }

    pub fn new_validation_error<T: AsRef<str>>(msg: T) -> Self {
        Self::ValidationError(msg.as_ref().to_string())
    }

    pub fn new_not_found<T: AsRef<str>>(resource: T) -> Self {
        Self::NotFoundError(resource.as_ref().to_string())
    }

    pub fn new_unknown<T: AsRef<str>>(msg: T) -> Self {
        Self::UnknownError(msg.as_ref().to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let error_json = serde_json::json!({
            "code": self.error_code(),
            "message": self.to_string(),
        });
        error_json.serialize(serializer)
    }
}

pub trait ResultExt<T, E> {
    fn with_context<C, F>(self, context: F) -> Result<T, AppError>
    where
        F: FnOnce() -> C,
        C: AsRef<str>;
}

use serde::Serialize;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    IOError(#[from] io::Error),
}

#[allow(dead_code)]
impl AppError {
    pub(crate) fn error_code(&self) -> i32 {
        match self {
            AppError::IOError(_) => 00001,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

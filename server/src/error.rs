//! Customized Server Errors

use aws_sdk_s3::error::ProvideErrorMetadata;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub struct S3Error(String);

impl S3Error {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn add_message(self, message: impl Into<String>) -> Self {
        Self(format!("{}: {}", message.into(), self.0))
    }
}

impl<T: ProvideErrorMetadata> From<T> for S3Error {
    fn from(value: T) -> Self {
        let err_code = value
            .code()
            .map(String::from)
            .unwrap_or("unknown code".into());

        let err_msg = value
            .message()
            .map(String::from)
            .unwrap_or("?".into());

        S3Error(format!("{}: {}", err_code, err_msg))
    }
}

impl Display for S3Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

use std::fmt::{Debug, Display};

use aws_sdk_s3::error::CopyObjectError;
use aws_sdk_s3::types::SdkError;

#[derive(Debug)]
pub enum Error {
    CSVParseError(csv::Error),
    S3CopyError(SdkError<CopyObjectError>),
}

impl Error {
    pub fn display(&self) -> Box<dyn Display + Send + Sync + '_> {
        match self {
            Error::CSVParseError(e) => Box::new(e),
            Error::S3CopyError(e) => Box::new(e),
        }
    }
}

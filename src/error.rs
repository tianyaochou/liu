use std::fmt::write;

use sqlx::Error as DBError;
use reqwest::Error as ReqError;
use feed_rs::parser::ParseFeedError;

#[derive(std::fmt::Debug)]
pub enum AppError {
    DBError(DBError),
    UpdateError(ReqError),
    FeedParseError(ParseFeedError)
}

pub type Result<T> = std::result::Result<T, AppError>;

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DBError(err) => write!(f, "{}", err),
            Self::UpdateError(err) => write!(f, "{}", err),
            Self::FeedParseError(err) => write!(f, "{}", err)
        }
    }
}

impl std::error::Error for AppError {}

impl From<DBError> for AppError {
    fn from(value: DBError) -> Self {
        Self::DBError(value)
    }
}

impl From<ReqError> for AppError {
    fn from(value: ReqError) -> Self {
        Self::UpdateError(value)
    }
}

impl From<ParseFeedError> for AppError {
    fn from(value: ParseFeedError) -> Self {
        Self::FeedParseError(value)
    }
}
//! # DeepSeek Error
//!
//! This module defines all errors that will happen in DeepSeek.

#[derive(Debug)]
/// The enum of the DeepSeek error type.
pub enum DeepSeekErrorType {
    /// The parameter of the request for deepseek api is wrong.
    RequestParamError,
    /// The reqeust send to deepseek is failed.
    RequestError,
    /// Error with api key
    ApiKeyError,
}

#[derive(Debug)]
/// The struct of the DeepSeek error.
pub struct DeepSeekError {
    error_type: DeepSeekErrorType,
    message: String,
}

impl DeepSeekError {
    /// Create a new DeepSeekError.
    pub fn new(error_type: DeepSeekErrorType, message: String) -> DeepSeekError {
        DeepSeekError {
            error_type,
            message,
        }
    }
}

impl std::fmt::Display for DeepSeekError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error_type {
            DeepSeekErrorType::RequestParamError => {
                write!(f, "RequestParamError: {}", self.message)
            }
            DeepSeekErrorType::RequestError => {
                write!(f, "RequestError: {}", self.message)
            }
            DeepSeekErrorType::ApiKeyError => {
                write!(f, "ApiKeyError: {}", self.message)
            }
        }
    }
}

pub type DeepSeekResult<T> = Result<T, DeepSeekError>;

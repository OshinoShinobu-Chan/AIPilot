//! # AI Node Error
//!
//! This module defines all errors that will happen in ai node.

pub mod deepseek_error;

use deepseek_error::DeepSeekError;

#[derive(Debug)]
/// The enum of the ai node error type.
pub enum AINodeErrorType {
    /// The error happens in DeepSeek.
    DeepSeekError(DeepSeekError),
}

#[derive(Debug)]
/// The struct of the ai node error.
pub struct AINodeError {
    error_type: AINodeErrorType,
    message: String,
}

impl AINodeError {
    /// Create a new AINodeError.
    pub fn new(error_type: AINodeErrorType, message: String) -> AINodeError {
        AINodeError {
            error_type,
            message,
        }
    }
}

impl std::fmt::Display for AINodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error_type {
            AINodeErrorType::DeepSeekError(e) => {
                write!(f, "DeepSeekError: {}\n{}", self.message, e)
            }
        }
    }
}

pub type AINodeResult<T> = Result<T, AINodeError>;

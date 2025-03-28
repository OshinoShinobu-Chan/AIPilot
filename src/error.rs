//! # Error
//!
//! This module is for a unified error handling. All errors that will happen in the program
//! should be defined here in a hierarchical way.

pub mod ai_node_error;

use ai_node_error::AINodeError;

#[derive(Debug)]
/// The enum of the error type.
pub enum PilotErrorType {
    /// The error happens in ai node
    AINodeErr(AINodeError),
}

#[derive(Debug)]
/// The struct is the root of all errors.
pub struct PilotError {
    error_type: PilotErrorType,
    message: String,
}

impl PilotError {
    /// Create a new PilotError.
    pub fn new(error_type: PilotErrorType, message: String) -> PilotError {
        PilotError {
            error_type,
            message,
        }
    }
}

impl std::fmt::Display for PilotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error_type {
            PilotErrorType::AINodeErr(ref e) => write!(f, "AINodeError: {}\n{}", self.message, e),
        }
    }
}

pub type PilotResult<T> = Result<T, PilotError>;

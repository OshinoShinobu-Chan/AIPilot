//! # AI node
//!
//! This node is for calling the AI service. It get the input and send a request to the api of the AI service.
//!
//! ## Input
//!
//! There are two inputs of the AI node:
//! 1. history: The history of the conversation.
//! 2. input: The input of the user.
//!
//! TODO: examples
//!
//! ## Output
//!
//! There is one output of the AI node:
//! 1. output: The output of the AI service.
//!
//! ## Supported AI Service
//! 1. DeepSeek

pub mod deepseek;

use deepseek::DeepSeekClient;

#[derive(Debug)]
/// The struct of one round of the chat.
pub struct Chat {
    role: String,
    content: String,
}

impl Chat {
    /// Create a new Chat.
    pub fn new(role: String, content: String) -> Chat {
        Chat { role, content }
    }
}

#[derive(Debug)]
/// The enum of the AI service.
pub enum AIService {
    DeepSeek { client: DeepSeekClient },
}

#[derive(Debug)]
/// The struct of the AI node.
pub struct AINode {
    service: AIService,
    histroy: Vec<Chat>,
    input: String,
}

impl AIService {
    /// Create a new AIService.
    pub fn new_deepseek(client: DeepSeekClient) -> AIService {
        AIService::DeepSeek { client }
    }
}

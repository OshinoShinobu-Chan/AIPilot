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
    DeepSeek {
        /// The url of the DeepSeek API.
        url: String,
        /// The api key of the DeepSeek API.
        api_key: String,
        /// The model of DeepSeek.
        model: String,
    },
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
    pub fn new_deepseek(url: String, api_key: String, model: String) -> AIService {
        AIService::DeepSeek {
            url,
            api_key,
            model,
        }
    }
}

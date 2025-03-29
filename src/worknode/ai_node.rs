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
    /// The AI service.
    service: AIService,
    /// The role of the ai assistant. Usually told by the role `system`,
    /// to tell the assistant what role it should play.
    /// For example, `system` role can be `You are a helpful assistant`.
    role: String,
    /// The history of the conversation.
    histroy: Vec<Chat>,
    /// The prefix of the prompt, which will be added in the beginning of the prompt.
    /// Usually used to give some background information to the assistant.
    /// For example, the pwd or the current time.
    prompt_prefix: String,
    /// The suffix of the prompt, which will be added in the end of the prompt.
    /// Usually used to give some restrictions of the output.
    /// For example, `Please answer in JSON format`.
    prompt_suffix: String,
    /// The input of the user.
    input: String,
}

impl AIService {
    /// Create a new AIService.
    pub fn new_deepseek(client: DeepSeekClient) -> AIService {
        AIService::DeepSeek { client }
    }
}

impl AINode {
    /// Create a new AINode.
    pub fn new(service: AIService) -> Self {
        AINode {
            service,
            role: String::new(),
            histroy: Vec::new(),
            prompt_prefix: String::new(),
            prompt_suffix: String::new(),
            input: String::new(),
        }
    }
    /// Set the role of teh assistant as builder.
    pub fn role(mut self, role: String) -> Self {
        self.role = role;
        self
    }
    /// Set the role of the assistant.
    pub fn set_role(&mut self, role: String) {
        self.role = role;
    }
    /// Get the role of the assistant.
    pub fn get_role(&self) -> &String {
        &self.role
    }
    /// Set the history of the conversation as builder.
    pub fn history(mut self, history: Vec<Chat>) -> Self {
        self.histroy = history;
        self
    }
    /// Set the history of the conversation.
    pub fn set_history(&mut self, history: Vec<Chat>) {
        self.histroy = history;
    }
    /// Get the history of the conversation.
    pub fn get_history(&self) -> &Vec<Chat> {
        &self.histroy
    }
    /// Push a chat to the history.
    pub fn push_history(&mut self, chat: Chat) {
        self.histroy.push(chat);
    }
    /// Set the prompt prefix as builder.
    pub fn prompt_prefix(mut self, prompt_prefix: String) -> Self {
        self.prompt_prefix = prompt_prefix;
        self
    }
    /// Set the prompt prefix.
    pub fn set_prompt_prefix(&mut self, prompt_prefix: String) {
        self.prompt_prefix = prompt_prefix;
    }
    /// Get the prompt prefix.
    pub fn get_prompt_prefix(&self) -> &String {
        &self.prompt_prefix
    }
    /// Set the prompt suffix as builder.
    pub fn prompt_suffix(mut self, prompt_suffix: String) -> Self {
        self.prompt_suffix = prompt_suffix;
        self
    }
    /// Set the prompt suffix.
    pub fn set_prompt_suffix(&mut self, prompt_suffix: String) {
        self.prompt_suffix = prompt_suffix;
    }
    /// Get the prompt suffix.
    pub fn get_prompt_suffix(&self) -> &String {
        &self.prompt_suffix
    }
    /// Set the input of the user as builder.
    pub fn input(mut self, input: String) -> Self {
        self.input = input;
        self
    }
    /// Set the input of the user.
    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }
    /// Get the input of the user.
    pub fn get_input(&self) -> &String {
        &self.input
    }
    /// Get the AI service.
    pub fn get_service(&self) -> &AIService {
        &self.service
    }
    /// Set the AI service.
    pub fn set_service(&mut self, service: AIService) {
        self.service = service;
    }
}

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

use crate::error::ai_node_error::{AINodeError, AINodeErrorType, AINodeResult};
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
    role: Option<String>,
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
            role: None,
            histroy: Vec::new(),
            prompt_prefix: String::new(),
            prompt_suffix: String::new(),
            input: String::new(),
        }
    }
    pub async fn execute(&mut self) -> AINodeResult<String> {
        match &mut self.service {
            AIService::DeepSeek { client } => {
                let prompt = format!(
                    "{}\n{}\n{}",
                    self.prompt_prefix, self.input, self.prompt_suffix
                );
                self.histroy
                    .push(Chat::new("user".to_string(), prompt.clone()));
                let response = client.send_request(&self.histroy).await.map_err(|e| {
                    AINodeError::new(
                        AINodeErrorType::DeepSeekError(e),
                        "Failed to send request to DeepSeek".to_string(),
                    )
                })?;
                let response_text = response["choices"][0]["message"]["content"].to_string();
                self.histroy.push(Chat::new(
                    "assistant".to_string(),
                    response_text.to_string(),
                ));

                Ok(response_text.to_string())
            }
        }
    }
    /// Set the role of teh assistant as builder.
    pub fn role(mut self, role: Option<String>) -> Self {
        let original_role_is_none = self.role.is_none();
        self.role = role;
        if self.role.is_none() {
            return self;
        }
        if original_role_is_none {
            self.histroy.insert(
                0,
                Chat::new("system".to_string(), self.role.clone().unwrap()),
            );
        } else {
            self.histroy[0] = Chat::new("system".to_string(), self.role.clone().unwrap());
        }
        self
    }
    /// Set the role of the assistant.
    pub fn set_role(&mut self, role: Option<String>) {
        self.role = role;
    }
    /// Get the role of the assistant.
    pub fn get_role(&self) -> &Option<String> {
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

#[cfg(test)]
mod test {
    use super::deepseek::{DeepSeekClient, DeepSeekModel, DEEPSEEK_API_URL};
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn deepseek_ai_node_execution() {
        let client = DeepSeekClient::new(DEEPSEEK_API_URL, DeepSeekModel::DeepseekChat)
            .api_key_from_file("./api_key.txt")
            .unwrap()
            .logprobs(true)
            .top_logprobs(Some(3));
        let mut ai_node = AINode::new(AIService::new_deepseek(client))
            .role(Some(
                "你是一个可爱的猫娘，请每一句话都使用猫娘的语气，并一定以“喵”结尾。".to_string(),
            ))
            .input("早上好".to_string());
        let result = ai_node.execute();
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(result);
        match result {
            Ok(output) => {
                println!("AI output: {}", output);
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }
    }
}

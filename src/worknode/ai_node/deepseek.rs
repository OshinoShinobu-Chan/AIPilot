//! # DeepSeek AI Node
//!
//! This module containes the supporting functions to use the DeepSeek api service.

use super::Chat;
use crate::error::ai_node_error::deepseek_error::{
    DeepSeekError, DeepSeekErrorType, DeepSeekResult,
};

use json::{object, JsonValue};

use reqwest::Response;

pub const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/chat/completions";

#[derive(Debug, Clone)]
pub enum ResponseFormat {
    /// The response format is text.
    Text,
    /// The response format is json.
    Json,
}

#[derive(Debug, Clone)]
pub struct StreamOption {
    /// If true, there will be an extra block containing the usage statistics.
    include_usage: bool,
}

#[derive(Debug, Clone)]
pub enum DeepSeekModel {
    DeepseekChat,
    DeepseekReasoner,
}

#[derive(Debug, Clone, Copy)]
/// The struct of usage statistics.
pub struct DeepSeekUsage {
    /// The number of tokens used in the response.
    completion_tokens: i64,
    /// The number of tokens used in the request.
    prompt_tokens: i64,
    /// The number of tokens used in the request that hits the cache.
    prompt_cache_hit_tokens: i64,
    /// The number of tokens used in the request that misses the cache.
    prompt_cache_miss_tokens: i64,
    /// The total number of tokens used.
    total_tokens: i64,
}

impl DeepSeekUsage {
    /// Create a new DeepSeekUsage.
    pub fn new() -> Self {
        DeepSeekUsage {
            completion_tokens: 0,
            prompt_tokens: 0,
            prompt_cache_hit_tokens: 0,
            prompt_cache_miss_tokens: 0,
            total_tokens: 0,
        }
    }
}

impl std::ops::Add for DeepSeekUsage {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        DeepSeekUsage {
            completion_tokens: self.completion_tokens + other.completion_tokens,
            prompt_tokens: self.prompt_tokens + other.prompt_tokens,
            prompt_cache_hit_tokens: self.prompt_cache_hit_tokens + other.prompt_cache_hit_tokens,
            prompt_cache_miss_tokens: self.prompt_cache_miss_tokens
                + other.prompt_cache_miss_tokens,
            total_tokens: self.total_tokens + other.total_tokens,
        }
    }
}

#[derive(Debug, Clone)]
/// The struct of the DeepSeek client.
pub struct DeepSeekClient {
    /// The url of the DeepSeek API.
    url: String,
    /// The api key of the DeepSeek API.
    api_key: Option<String>,
    /// The model of DeepSeek.
    model: DeepSeekModel,
    /// The panalty of frequency, if this value is larger than 0, deepseek will get panalty
    /// for the frequency of the content that has been generated.
    ///
    /// The value should be between -2.0 and 2.0, default is 0.0.
    frequency_panalty: Option<f64>,
    /// The maximum tokens of the response, default is 4096.
    max_tokens: Option<i32>,
    /// The panalty of presence, if this value is larger than 0, deepseek will get panalty
    /// for the presence of the content that has been generated.
    ///
    /// The value should be between -2.0 and 2.0, default is 0.0.
    presence_penalty: Option<f64>,
    /// The format of the response, default is text.
    response_format: Option<ResponseFormat>,
    /// Whether use stream to send the request, default is false.
    stream: Option<bool>,
    /// The stream option.
    stream_option: Option<StreamOption>,
    /// The temperature of the response. With a higher temperature, the model will be more
    /// random.
    /// Don't use this with top p.
    /// The value should be between 0 and 2, default is 1.
    temperature: Option<f64>,
    /// The top p of the response. With a higher top p, the model will be more random.
    /// Don't use this with temperature.
    /// The value should be between 0 and 1, default is 1.
    top_p: Option<f64>,
    // tools： not supported yet
    // tool choice: not supported yet
    /// Whether use logprobs in the response, default is false.
    logprobs: bool,
    /// Return the top n tokens in every position. Can only be used when logprobs is true.
    top_logprobs: Option<i32>,
    /// The total usage statistics of the client.
    total_usage: DeepSeekUsage,
    /// The last usage statistics of the client.
    last_usage: DeepSeekUsage,
}

impl DeepSeekClient {
    pub fn new(url: &str, model: DeepSeekModel) -> DeepSeekClient {
        DeepSeekClient {
            url: url.to_string(),
            api_key: None,
            model,
            frequency_panalty: None,
            max_tokens: None,
            presence_penalty: None,
            response_format: None,
            stream: None,
            stream_option: None,
            temperature: None,
            top_p: None,
            logprobs: false,
            top_logprobs: None,
            total_usage: DeepSeekUsage::new(),
            last_usage: DeepSeekUsage::new(),
        }
    }
    /// Get a request string from the client and history chats, and send the request
    /// to the DeepSeek API. This function is asynchronous.
    /// The request string is in json format.
    /// This function garantees that the request consist the response message.
    pub async fn send_request(&mut self, chats: &Vec<Chat>) -> DeepSeekResult<JsonValue> {
        if !self.check_params() {
            return Err(DeepSeekError::new(
                DeepSeekErrorType::RequestParamError,
                "The parameters are not valid.".to_string(),
            ));
        }
        let request = self.into_request_string(Self::chats_to_json(chats));
        // api key is already checked in check_params, so unwrap is safe here
        let api_key = self.api_key.clone().unwrap();
        let response = Self::send_request_raw(request, api_key).await?;
        let response_text = json::parse(
            response
                .text()
                .await
                .map_err(|e| {
                    DeepSeekError::new(
                        DeepSeekErrorType::RequestError,
                        format!("Failed to read response text. {}", e),
                    )
                })?
                .as_str(),
        )
        .map_err(|e| {
            DeepSeekError::new(
                DeepSeekErrorType::RequestError,
                format!("Failed to parse response text. {}", e),
            )
        })?;
        // check response
        if response_text["choices"][0]["message"]["content"].is_null() {
            return Err(DeepSeekError::new(
                DeepSeekErrorType::ResponseError,
                "The response format is not valid.".to_string(),
            ));
        } else if response_text["choices"][0]["message"]["content"].is_empty() {
            return Err(DeepSeekError::new(
                DeepSeekErrorType::ResponseError,
                "The response is empty.".to_string(),
            ));
        }
        // dump the usage statistics
        let usage = response_text["usage"].clone();
        if usage.is_null() {
            return Err(DeepSeekError::new(
                DeepSeekErrorType::ResponseError,
                "The response does not contain usage statistics.".to_string(),
            ));
        } else if usage.is_empty() {
            return Err(DeepSeekError::new(
                DeepSeekErrorType::ResponseError,
                "The usage statistics is empty.".to_string(),
            ));
        }
        self.last_usage = DeepSeekUsage {
            completion_tokens: usage["completion_tokens"]
                .as_i64()
                .ok_or(DeepSeekError::new(
                    DeepSeekErrorType::ResponseError,
                    "The response does not contain completion tokens.".to_string(),
                ))?,
            prompt_tokens: usage["prompt_tokens"].as_i64().ok_or(DeepSeekError::new(
                DeepSeekErrorType::ResponseError,
                "The response does not contain prompt tokens.".to_string(),
            ))?,
            prompt_cache_hit_tokens: usage["prompt_cache_hit_tokens"].as_i64().ok_or(
                DeepSeekError::new(
                    DeepSeekErrorType::ResponseError,
                    "The response does not contain prompt cache hit tokens.".to_string(),
                ),
            )?,
            prompt_cache_miss_tokens: usage["prompt_cache_miss_tokens"].as_i64().ok_or(
                DeepSeekError::new(
                    DeepSeekErrorType::ResponseError,
                    "The response does not contain prompt cache miss tokens.".to_string(),
                ),
            )?,
            total_tokens: usage["total_tokens"].as_i64().ok_or(DeepSeekError::new(
                DeepSeekErrorType::ResponseError,
                "The response does not contain total tokens.".to_string(),
            ))?,
        };
        self.total_usage = self.total_usage + self.last_usage;
        Ok(response_text)
    }
    /// Send the request to the DeepSeek API. This function is asynchronous.
    async fn send_request_raw(request: String, api_key: String) -> DeepSeekResult<Response> {
        let client = reqwest::Client::new();
        let response = client
            .post(DEEPSEEK_API_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .body(request)
            .send()
            .await
            .map_err(|_| {
                DeepSeekError::new(
                    DeepSeekErrorType::RequestError,
                    "Failed to send request.".to_string(),
                )
            })?;
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(DeepSeekError::new(
                DeepSeekErrorType::RequestError,
                format!(
                    "Request failed with status: {}, {}",
                    response.status(),
                    response
                        .text()
                        .await
                        .map(|r| json::parse(&r)
                            .map(|r| r["error"]["message"].to_string())
                            .unwrap_or("Failed to parse error message".to_string(),))
                        .unwrap_or("Failed to read error message".to_string())
                ),
            ))
        }
    }
    /// Convert the chats to json format.
    fn chats_to_json(chats: &Vec<Chat>) -> JsonValue {
        let mut json_chats = Vec::new();
        for chat in chats {
            json_chats.push(object! {
                content: chat.content.clone(),
                role: chat.role.clone(),
            });
        }
        json::JsonValue::Array(json_chats)
    }
    /// Convert the client and the chats to json format.
    fn into_request_string(&self, msg: JsonValue) -> String {
        object! {
            messages: msg,
            model: self.model.to_string(),
            frequency_panalty: self.frequency_panalty.unwrap_or(Self::default_frequency_panalty()),
            max_tokens: self.max_tokens.unwrap_or(Self::default_max_tokens()),
            presence_penalty: self.presence_penalty.unwrap_or(Self::default_presence_penalty()),
            response_format: object! {
                "type": self.response_format.clone().unwrap_or(Self::default_response_format()).to_string(),
            },
            stop: json::JsonValue::Null,
            stream: self.stream.unwrap_or(Self::default_stream()),
            stream_options: if let Some(stream_option) = self.stream_option.clone() {
                object! {
                    include_usage: stream_option.include_usage,
                }
            } else {
                json::JsonValue::Null
            },
            temperature: self.temperature.unwrap_or(Self::default_temperature()),
            top_p: self.top_p.unwrap_or(Self::default_top_p()),
            tools: json::JsonValue::Null,
            tool_choice: "none",
            logprobs: self.logprobs,
            top_logprobs: self.top_logprobs,
        }.dump()
    }
    /// Check if the parameters are valid, including:
    /// - frequency_panalty
    /// - max_tokens
    /// - presence_penalty
    /// - stream_option
    /// - temperature
    /// - top_p
    /// - top_logprobs
    /// - api_key
    pub fn check_params(&self) -> bool {
        self.check_frequency_panalty()
            && self.check_max_tokens()
            && self.check_presence_penalty()
            && self.check_stream_option()
            && self.check_temperature()
            && self.check_top_p()
            && self.check_top_logprobs()
            && self.api_key.is_some()
    }
    pub fn get_url(&self) -> &str {
        &self.url
    }
    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }
    pub fn api_key_from_env(mut self) -> DeepSeekResult<Self> {
        self.api_key = Some(std::env::var("API_KEY").map_err(|_| {
            DeepSeekError::new(
                DeepSeekErrorType::ApiKeyError,
                "Environment variable API_KEY not found.".to_string(),
            )
        })?);
        Ok(self)
    }
    pub fn api_key_from_file(mut self, file: &str) -> DeepSeekResult<Self> {
        let api_key = std::fs::read_to_string(file);
        match api_key {
            Ok(api_key) => {
                self.api_key = Some(api_key);
                Ok(self)
            }
            Err(_) => Err(DeepSeekError::new(
                DeepSeekErrorType::ApiKeyError,
                "Can't read the api key file.".to_string(),
            )),
        }
    }
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
    pub fn set_api_key(&mut self, api_key: Option<String>) {
        self.api_key = api_key;
    }
    pub fn get_model(&self) -> &DeepSeekModel {
        &self.model
    }
    pub fn set_model(&mut self, model: DeepSeekModel) {
        self.model = model;
    }
    pub fn frequency_panalty(mut self, frequency_panalty: Option<f64>) -> Self {
        self.frequency_panalty = frequency_panalty;
        self
    }
    pub fn get_frequency_panalty(&self) -> Option<f64> {
        self.frequency_panalty
    }
    pub fn set_frequency_panalty(&mut self, frequency_panalty: Option<f64>) {
        self.frequency_panalty = frequency_panalty;
    }
    pub fn default_frequency_panalty() -> f64 {
        0.0
    }
    pub fn check_frequency_panalty(&self) -> bool {
        if let Some(frequency_panalty) = self.frequency_panalty {
            if frequency_panalty < -2.0 || frequency_panalty > 2.0 {
                return false;
            }
        }
        true
    }
    pub fn max_tokens(mut self, max_tokens: Option<i32>) -> Self {
        self.max_tokens = max_tokens;
        self
    }
    pub fn get_max_tokens(&self) -> Option<i32> {
        self.max_tokens
    }
    pub fn set_max_tokens(&mut self, max_tokens: Option<i32>) {
        self.max_tokens = max_tokens;
    }
    pub fn default_max_tokens() -> i32 {
        4096
    }
    pub fn check_max_tokens(&self) -> bool {
        if let Some(max_tokens) = self.max_tokens {
            if max_tokens < 1 || max_tokens > 8192 {
                return false;
            }
        }
        true
    }
    pub fn presence_penalty(mut self, presence_penalty: Option<f64>) -> Self {
        self.presence_penalty = presence_penalty;
        self
    }
    pub fn get_presence_penalty(&self) -> Option<f64> {
        self.presence_penalty
    }
    pub fn set_presence_penalty(&mut self, presence_penalty: Option<f64>) {
        self.presence_penalty = presence_penalty;
    }
    pub fn default_presence_penalty() -> f64 {
        0.0
    }
    pub fn check_presence_penalty(&self) -> bool {
        if let Some(presence_penalty) = self.presence_penalty {
            if presence_penalty < -2.0 || presence_penalty > 2.0 {
                return false;
            }
        }
        true
    }
    pub fn response_format(mut self, response_format: Option<ResponseFormat>) -> Self {
        self.response_format = response_format;
        self
    }
    pub fn get_response_format(&self) -> Option<ResponseFormat> {
        self.response_format.clone()
    }
    pub fn set_response_format(&mut self, response_format: Option<ResponseFormat>) {
        self.response_format = response_format;
    }
    pub fn default_response_format() -> ResponseFormat {
        ResponseFormat::Text
    }
    pub fn stream(mut self, stream: Option<bool>) -> Self {
        self.stream = stream;
        self
    }
    pub fn get_stream(&self) -> Option<bool> {
        self.stream
    }
    pub fn set_stream(&mut self, stream: Option<bool>) {
        self.stream = stream;
    }
    pub fn default_stream() -> bool {
        false
    }
    pub fn stream_option(mut self, stream_option: Option<StreamOption>) -> Self {
        self.stream_option = stream_option;
        self
    }
    pub fn get_stream_option(&self) -> Option<StreamOption> {
        self.stream_option.clone()
    }
    pub fn set_stream_option(&mut self, stream_option: Option<StreamOption>) {
        self.stream_option = stream_option;
    }
    pub fn check_stream_option(&self) -> bool {
        self.stream_option.is_none()
            || match self.stream {
                Some(stream) => stream,
                None => Self::default_stream(),
            }
    }
    pub fn temperature(mut self, temperature: Option<f64>) -> Self {
        self.temperature = temperature;
        self
    }
    pub fn get_temperature(&self) -> Option<f64> {
        self.temperature
    }
    pub fn set_temperature(&mut self, temperature: Option<f64>) {
        self.temperature = temperature;
    }
    pub fn default_temperature() -> f64 {
        1.0
    }
    pub fn check_temperature(&self) -> bool {
        if let Some(temperature) = self.temperature {
            if temperature < 0.0 || temperature > 2.0 {
                return false;
            }
        }
        true
    }
    pub fn top_p(mut self, top_p: Option<f64>) -> Self {
        self.top_p = top_p;
        self
    }
    pub fn get_top_p(&self) -> Option<f64> {
        self.top_p
    }
    pub fn set_top_p(&mut self, top_p: Option<f64>) {
        self.top_p = top_p;
    }
    pub fn default_top_p() -> f64 {
        1.0
    }
    pub fn check_top_p(&self) -> bool {
        if let Some(top_p) = self.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                return false;
            }
        }
        true
    }
    pub fn logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = logprobs;
        self
    }
    pub fn get_logprobs(&self) -> bool {
        self.logprobs
    }
    pub fn set_logprobs(&mut self, logprobs: bool) {
        self.logprobs = logprobs;
    }
    pub fn top_logprobs(mut self, top_logprobs: Option<i32>) -> Self {
        self.top_logprobs = top_logprobs;
        self
    }
    pub fn get_top_logprobs(&self) -> Option<i32> {
        self.top_logprobs
    }
    pub fn set_top_logprobs(&mut self, top_logprobs: Option<i32>) {
        self.top_logprobs = top_logprobs;
    }
    pub fn check_top_logprobs(&self) -> bool {
        self.logprobs || self.top_logprobs.is_none()
    }
}

use crate::error::ai_node_error::{AINodeError, AINodeErrorType, AINodeResult};
impl super::AINode {
    pub(super) async fn deepseek_execute(&mut self) -> AINodeResult<String> {
        let client = match &mut self.service {
            super::AIService::DeepSeek { client: client } => client,
            _ => {
                unreachable!()
            }
        };
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

impl std::fmt::Display for DeepSeekModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeepSeekModel::DeepseekChat => write!(f, "deepseek-chat"),
            DeepSeekModel::DeepseekReasoner => write!(f, "deepseek-reasoner"),
        }
    }
}

impl std::fmt::Display for ResponseFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseFormat::Text => write!(f, "text"),
            ResponseFormat::Json => write!(f, "json"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::runtime::Runtime;
    #[test]
    fn build_deepseek_client_simpl() {
        let deepseek_client = DeepSeekClient::new(DEEPSEEK_API_URL, DeepSeekModel::DeepseekChat)
            .api_key_from_file("./api_key.txt")
            .unwrap();
        assert!(deepseek_client.check_params());
    }

    #[test]
    fn build_deepseek_client() {
        let deepseek_client = DeepSeekClient::new(DEEPSEEK_API_URL, DeepSeekModel::DeepseekChat)
            .api_key_from_env()
            .unwrap()
            .frequency_panalty(Some(0.5))
            .max_tokens(Some(2048))
            .presence_penalty(Some(0.5))
            .response_format(Some(ResponseFormat::Json))
            .stream(Some(true))
            .stream_option(Some(StreamOption {
                include_usage: true,
            }))
            .temperature(Some(0.5))
            .top_p(Some(0.5))
            .logprobs(true)
            .top_logprobs(Some(10));
        assert!(deepseek_client.check_params());
    }

    #[test]
    fn into_request_string() {
        let deepseek_client = DeepSeekClient::new(DEEPSEEK_API_URL, DeepSeekModel::DeepseekChat)
            .api_key_from_file("./api_key.txt")
            .unwrap()
            .frequency_panalty(Some(0.5))
            .max_tokens(Some(2048))
            .presence_penalty(Some(0.5))
            .response_format(Some(ResponseFormat::Text))
            .stream(Some(false))
            .temperature(Some(0.5))
            .top_p(Some(0.5))
            .logprobs(true)
            .top_logprobs(Some(10));
        let request_string = json::parse(
            deepseek_client
                .into_request_string(
                    json::parse(
                        r#"[
                                {
                                "content": "You are a helpful assistant",
                                "role": "system"
                                },
                                {
                                "content": "Hi",
                                "role": "user"
                                }
                            ]"#,
                    )
                    .unwrap(),
                )
                .as_str(),
        )
        .unwrap();

        let example = json::parse(
            r#"
    {
  "messages": [
    {
      "content": "You are a helpful assistant",
      "role": "system"
    },
    {
      "content": "Hi",
      "role": "user"
    }
  ],
  "model": "deepseek-chat",
  "frequency_penalty": 0.5,
  "max_tokens": 2048,
  "presence_penalty": 0.5,
  "response_format": {
    "type": "text"
  },
  "stop": null,
  "stream": false,
  "stream_options": null,
  "temperature": 0.5,
  "top_p": 0.5,
  "tools": null,
  "tool_choice": "none",
  "logprobs": true,
  "top_logprobs": 10
}"#,
        )
        .unwrap();
        assert_eq!(request_string.as_str(), example.as_str());
    }

    #[test]
    fn send_request_simple() {
        let rt = Runtime::new().unwrap();
        let mut deepseek_client =
            DeepSeekClient::new(DEEPSEEK_API_URL, DeepSeekModel::DeepseekChat)
                .api_key_from_file("./api_key.txt")
                .unwrap();
        let chats = vec![
            Chat::new(
                "system".to_string(),
                "You are a helpful assistant".to_string(),
            ),
            Chat::new("user".to_string(), "Hi".to_string()),
        ];
        let response = rt.block_on(deepseek_client.send_request(&chats));
        match response {
            Ok(response) => {
                println!("Response: {}", response);
            }
            Err(e) => {
                println!("{}", e);
                panic!("Failed to send request");
            }
        }
    }

    #[test]
    /// Test the send_request function with complex parameters.
    fn send_request_complex() {
        let rt = Runtime::new().unwrap();
        let mut deepseek_client =
            DeepSeekClient::new(DEEPSEEK_API_URL, DeepSeekModel::DeepseekChat)
                .api_key_from_file("./api_key.txt")
                .unwrap()
                .logprobs(true)
                .top_logprobs(Some(3));
        let chats = vec![
            Chat::new(
                "system".to_string(),
                "You are a helpful assistant".to_string(),
            ),
            Chat::new("user".to_string(), "Hi".to_string()),
        ];
        let response = rt.block_on(deepseek_client.send_request(&chats));
        match response {
            Ok(response) => {
                println!("Response: {}", response);
            }
            Err(e) => {
                println!("{}", e);
                panic!("Failed to send request");
            }
        }
    }
}

use std::fmt;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ApiEndpoint {
    OpenAIEndpoint,
    FreeEndpoint,
}

impl fmt::Display for ApiEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiEndpoint::OpenAIEndpoint => write!(f, "https://api.openai.com/v1/chat/completions"),
            ApiEndpoint::FreeEndpoint => {
                write!(f, "https://free.churchless.tech/v1/chat/completions")
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OAIModel {
    GPT3Turbo,
    GPT3_5Turbo0301,
    GPT4,
    GPT4_32K,
    GPT4_0314,
    GPT4_32K0314,
}

impl fmt::Display for OAIModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OAIModel::GPT3Turbo => write!(f, "gpt-3.5-turbo"),
            OAIModel::GPT3_5Turbo0301 => write!(f, "gpt-3.5-turbo-0301"),
            OAIModel::GPT4 => write!(f, "gpt-4"),
            OAIModel::GPT4_32K => write!(f, "gpt-4-32k"),
            OAIModel::GPT4_0314 => write!(f, "gpt-4-0314"),
            OAIModel::GPT4_32K0314 => write!(f, "gpt-4-32k-0314"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Deserialize, Clone)]
pub struct ChatCompletionChoice {
    pub index: u64,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Deserialize, Clone)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Deserialize, Clone)]
pub struct OAIResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: Option<Usage>,
}

#[derive(Serialize, Builder, Debug, Clone)]
#[builder(pattern = "owned")]
#[builder(setter(strip_option, into))]
pub struct OAIRequest {
    pub(crate) model: OAIModel,
    messages: Vec<Message>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u64>,
}

impl OAIRequest {
    pub fn builder(
        model: impl Into<OAIModel>,
        messages: impl Into<Vec<Message>>,
    ) -> OAIRequestBuilder {
        OAIRequestBuilder::create_empty()
            .model(model)
            .messages(messages)
    }
}

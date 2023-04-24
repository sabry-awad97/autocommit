use anyhow::{anyhow, Context, Error};
use derive_builder::Builder;
use dotenv::dotenv;
use reqwest::{header::HeaderValue, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt;

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

pub struct OAIConfig {
    end_point: ApiEndpoint,
    openai_api_key: String,
}

impl OAIConfig {
    fn new(end_point: ApiEndpoint, openai_api_key: String) -> Self {
        Self {
            end_point,
            openai_api_key,
        }
    }
}

struct OpenAI {
    pub config: OAIConfig,
}

impl OpenAI {
    fn new(config: OAIConfig) -> Self {
        Self { config }
    }

    async fn send_request(&mut self, chat_request: &OAIRequest) -> Result<OAIResponse, Error> {
        let mut end_point = &self.config.end_point;

        let model = &chat_request.model;

        if model.to_string().contains("gpt-4") {
            end_point = &ApiEndpoint::FreeEndpoint;
        }

        let url = &end_point.to_string();

        let response = reqwest::Client::new()
            .post(url)
            .header(
                reqwest::header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )
            .bearer_auth(&self.config.openai_api_key)
            .json(&chat_request)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {}", url))?;

        match response.status() {
            StatusCode::OK => {
                let response = response
                    .json::<OAIResponse>()
                    .await
                    .map_err(|err| anyhow!("Failed to decode json response: {}", err))?;
                Ok(response)
            }
            StatusCode::TOO_MANY_REQUESTS => Err(anyhow!("Rate limit exceeded")),
            _ => Err(anyhow!("Unexpected HTTP response: {:?}", response.status())),
        }
    }

    async fn create_chat_completion(
        &mut self,
        model_name: OAIModel,
        messages: impl Into<Vec<Message>>,
        max_tokens: usize,
    ) -> Result<OAIResponse, Error> {
        let chat_request = OAIRequest::builder(model_name, messages)
            .max_tokens::<u64>(max_tokens.try_into().unwrap())
            .temperature(0.5)
            .top_p(0.1)
            .build()?;

        let response = &self
            .send_request(&chat_request)
            .await
            .map_err(|err| anyhow!("Failed to generate code: {}", err))?;

        Ok(response.to_owned())
    }
}

struct Generator {
    openai: OpenAI,
}

impl Generator {
    fn new(api_key: &str) -> Self {
        let config: OAIConfig = OAIConfig::new(ApiEndpoint::FreeEndpoint, api_key.to_string());
        let openai: OpenAI = OpenAI::new(config);
        Self { openai }
    }

    async fn generate(&mut self, prompt: &[Message]) -> anyhow::Result<String> {
        let response = self
            .openai
            .create_chat_completion(OAIModel::GPT4, prompt, 196)
            .await?;

        let result = response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow!("No message returned"))?;

        Ok(result)
    }
}

pub async fn generate_message(prompt: &[Message]) -> anyhow::Result<String> {
    dotenv().ok();
    let openai_api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("Please set OPENAI_API_KEY environment variable"))?;

    let mut gen = Generator::new(&openai_api_key);
    gen.generate(prompt).await
}

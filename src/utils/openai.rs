use anyhow::{anyhow, Error};
use derive_builder::Builder;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr, time::Duration};
use surf::{Client, StatusCode};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OAIModel {
    #[serde(rename = "gpt-3.5-turbo")]
    GPT3Turbo,
    GPT3_5Turbo0301,
    #[serde(rename = "gpt-4")]
    GPT4,
    GPT4_32K,
    GPT4_0314,
    GPT4_32K0314,
}

impl std::str::FromStr for OAIModel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gpt-3.5-turbo" => Ok(OAIModel::GPT3Turbo),
            "gpt-3.5-turbo-0301" => Ok(OAIModel::GPT3_5Turbo0301),
            "gpt-4" => Ok(OAIModel::GPT4),
            "gpt-4-32k" => Ok(OAIModel::GPT4_32K),
            "gpt-4-0314" => Ok(OAIModel::GPT4_0314),
            "gpt-4-32k-0314" => Ok(OAIModel::GPT4_32K0314),
            _ => Err(format!("Invalid OpenAI Model: {}", s)),
        }
    }
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

impl Message {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self { role, content }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ChatCompletionChoice {
    pub index: u64,
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Deserialize, Clone, Debug)]
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
    api_host: String,
    openai_api_key: String,
}

impl OAIConfig {
    fn new(api_host: impl Into<String>, openai_api_key: impl Into<String>) -> Self {
        Self {
            api_host: api_host.into(),
            openai_api_key: openai_api_key.into(),
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
        let url = self.config.api_host.to_string() + "/v1/chat/completions";
        let mut retries = 0;

        loop {
            let client = Client::new();
            let mut response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.config.openai_api_key),
                )
                .body_json(chat_request)
                .map_err(|err| anyhow!("Failed to send request to api: {}", err))?
                .await
                .map_err(|err| anyhow!("Failed to send request to api: {}", err))?;

            debug!("Request sent to {}", url);
            match response.status() {
                StatusCode::Ok => {
                    let response = response
                        .body_json::<OAIResponse>()
                        .await
                        .map_err(|err| anyhow!("Failed to decode json response: {}", err))?;
                    info!("Request successful {:#?}", response);
                    return Ok(response);
                }
                StatusCode::TooManyRequests => {
                    let error = anyhow!("Rate limit exceeded");
                    return Err(error);
                }
                status_code if retries < 5 => {
                    retries += 1;
                    warn!(
                        "Unexpected HTTP response: {:?} - Retrying ({}/{})...",
                        status_code, retries, 5
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                status_code => {
                    let error_message = response
                        .body_string()
                        .await
                        .unwrap_or_else(|err| format!("Unknown error: {}", err));
                    let error = anyhow!(
                        "Unexpected HTTP response: {:?} - {}",
                        status_code,
                        error_message
                    );
                    return Err(error);
                }
            }
        }
    }

    async fn create_chat_completion(
        &mut self,
        model_name: OAIModel,
        messages: impl Into<Vec<Message>>,
    ) -> Result<OAIResponse, Error> {
        info!("Creating chat completion with model: {}", model_name);

        let chat_request = OAIRequest::builder(model_name, messages)
            .temperature(0.5)
            .top_p(0.1)
            .build()?;

        debug!("Request body: {:?}", chat_request);

        let response = &self
            .send_request(&chat_request)
            .await
            .map_err(|err| anyhow!("Failed to generate code: {}", err))?;
        info!("Response: {:?}", response);
        Ok(response.to_owned())
    }
}

struct Generator {
    openai: OpenAI,
}

impl Generator {
    fn new(api_key: &str, api_host: &str) -> Self {
        let config: OAIConfig = OAIConfig::new(api_host, api_key);
        let openai: OpenAI = OpenAI::new(config);
        Self { openai }
    }

    async fn generate(&mut self, prompt: &[Message], model_name: &str) -> anyhow::Result<String> {
        let model = OAIModel::from_str(model_name).map_err(|err| anyhow!(err))?;

        let response = self.openai.create_chat_completion(model, prompt).await?;

        let result = response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow!("No message returned"))?;
        info!("Generated message: {}", result);
        Ok(result)
    }
}

pub async fn generate_message(
    prompt: &[Message],
    open_ai_api_key: &str,
    api_host: &str,
    model: &Option<String>,
) -> anyhow::Result<String> {
    let mut gen = Generator::new(open_ai_api_key, api_host);
    gen.generate(prompt, &model.clone().unwrap_or("gpt-3.5-turbo".to_owned()))
        .await
}

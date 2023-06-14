use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chatgpt::client::ChatGPT;
use chatgpt::config::{ChatGPTEngine, ModelConfiguration, OldChatGPTEngine, OldModelConfiguration};
use chatgpt::prelude::ChatMessage;
use chatgpt::types::{CompletionResponse, Role};
use chrono::Utc;
use futures::executor::block_on;
use reqwest::{blocking::Client, Url};

use crate::error::Error;
use crate::languages::Language;
use crate::translators::chatbot;
use crate::translators::context::{get_gpt_context, Context};
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorContext,
};

/// Chatgpt models like GPT-3
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum ChatGPTModel {
    /// Supported models: GPT-3, GPT-3.5-Turbo, GPT-4
    GPT3,
    #[default]
    Gpt35Turbo,
    GPT4,
}

impl FromStr for ChatGPTModel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Self::GPT3),
            "GPT3" => Ok(Self::GPT3),
            "GPT3.5-Turbo" => Ok(Self::Gpt35Turbo),
            "GPT4" => Ok(Self::GPT4),
            _ => Err(Error::new_option(format!(
                "Invalid model: {}. Supported models: GPT-3, GPT-3.5-Turbo, GPT-4",
                s
            ))),
        }
    }
}

/// Translator struct that contains the request data
#[derive(Debug, Clone)]
pub struct ChatGPTTranslator {
    client: ChatGPT,
    last_request: Arc<Mutex<i64>>,
    wait_time: i64,
}

impl TranslatorContext for ChatGPTTranslator {
    fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
        context: &[Context],
    ) -> Result<TranslationOutput, Error> {
        let v = self.translate_vec(client, &[query.to_string()], from, to, context)?;
        Ok(TranslationOutput {
            text: v.text.join("\n"),
            lang: v.lang,
        })
    }

    fn translate_vec(
        &self,
        _: &Client,
        query: &[String],
        _: Option<Language>,
        to: &Language,
        context: &[Context],
    ) -> Result<TranslationVecOutput, Error> {
        let time = {
            self.last_request
                .lock()
                .map(|v| *v)
                .map_err(|v| Error::new("Failed to lock last request", v))
        }?;
        let wait = self.wait_time - (Utc::now().timestamp() - time);
        if wait > 0 {
            std::thread::sleep(Duration::from_secs(wait as u64));
        }
        let con = get_gpt_context(context);
        let q_s = chatbot::generate_query(query, &to.to_name_str()?, con)?;

        let response: CompletionResponse = block_on(async {
            self.client
                .send_history(&vec![ChatMessage {
                    role: Role::System,
                    content: "You are a professional translator who will follow the required format for translation.".into(),
                }, ChatMessage {
                    role: Role::User,
                    content: q_s,
                }]).await.map_err(|v| Error::new("Failed to send history", v))
        })?;

        let message = response.message().content.to_string();
        chatbot::process_result(message, query)
    }
}

impl ChatGPTTranslator {
    /// Creates a new translator
    pub fn new(
        model: &ChatGPTModel,
        token: &str,
        proxy: &str,
        old_proxy: &str,
        temperature: f32,
        wait_time: &Duration,
    ) -> Result<Self, Error> {
        let url = Url::from_str(match proxy {
            "" => "https://api.openai.com/v1/chat/completions",
            _ => proxy,
        })
        .map_err(|e| Error::new("Failed to parse url", e))?;
        let oldurl = Url::from_str(match old_proxy {
            "" => "https://api.openai.com/v1/completions",
            _ => old_proxy,
        })
        .map_err(|e| Error::new("Failed to parse url", e))?;
        let client = match model {
            ChatGPTModel::GPT3 => {
                let config = OldModelConfiguration {
                    engine: OldChatGPTEngine::text_davinci_003,
                    temperature,
                    max_tokens: 2000,
                    top_p: 0.0,
                    presence_penalty: 0.0,
                    frequency_penalty: 0.0,
                    reply_count: 1,
                    api_url: oldurl,
                    logprobs: None,
                    stop: "\n".to_string(),
                };
                ChatGPT::new_with_old_config(token, config).map_err(|v| v.to_string())
            }
            ChatGPTModel::Gpt35Turbo | ChatGPTModel::GPT4 => {
                let model = match model {
                    ChatGPTModel::Gpt35Turbo => ChatGPTEngine::Gpt35Turbo,
                    ChatGPTModel::GPT4 => ChatGPTEngine::Gpt4,
                    _ => ChatGPTEngine::Gpt35Turbo,
                };
                let config = ModelConfiguration {
                    engine: model,
                    temperature,
                    top_p: 0.0,
                    presence_penalty: 0.0,
                    frequency_penalty: 0.0,
                    reply_count: 1,
                    api_url: url,
                };
                ChatGPT::new_with_config(token, config).map_err(|v| v.to_string())
            }
        }
        .map_err(|e| Error::new("Failed to initialize chatgpt", e))?;
        Ok(Self {
            client,
            last_request: Arc::new(Mutex::new(0)),
            wait_time: wait_time.as_millis() as i64,
        })
    }
}

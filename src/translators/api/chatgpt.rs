use async_trait::async_trait;
use chatgpt::client::ChatGPT;
use chatgpt::config::{ChatGPTEngine, ModelConfiguration, OldChatGPTEngine, OldModelConfiguration};
use chatgpt::prelude::ChatMessage;
use chatgpt::types::{CompletionResponse, Role};
use reqwest::Client;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::chatbot;
use crate::translators::context::{get_gpt_context, Context};
use crate::translators::translator_structrue::{
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

/// Translator struct that contains the request data
#[derive(Debug, Clone)]
pub struct ChatGPTTranslator {
    client: ChatGPT,
}

#[async_trait]
impl TranslatorContext for ChatGPTTranslator {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
        context: &[Context],
    ) -> Result<TranslationOutput, Error> {
        let v = self
            .translate_vec(client, &[query.to_string()], from, to, context)
            .await?;
        Ok(TranslationOutput {
            text: v.text.join("\n"),
            lang: v.lang,
        })
    }

    async fn translate_vec(
        &self,
        _: &Client,
        query: &[String],
        _: Option<Language>,
        to: &Language,
        context: &[Context],
    ) -> Result<TranslationVecOutput, Error> {
        let con = get_gpt_context(context);
        let q_s = chatbot::generate_query(query, &to.to_name_str()?, con)?;

        let response: CompletionResponse = self.client
            .send_history(&vec![ChatMessage {
                role: Role::System,
                content: "You are a professional translator who will follow the required format for translation.".into(),
            }, ChatMessage {
                role: Role::User,
                content: q_s,
            }], )
            .await.map_err(|v| Error::new("Failed to send history", v))?;
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
    ) -> Result<Self, Error> {
        let url = match proxy {
            "" => "https://api.openai.com/v1/chat/completions".to_string(),
            _ => proxy.to_string(),
        };
        let oldurl = match old_proxy {
            "" => "https://api.openai.com/v1/completions".to_string(),
            _ => old_proxy.to_string(),
        };
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
        Ok(Self { client })
    }
}

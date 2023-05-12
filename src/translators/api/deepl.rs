use std::str::FromStr;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::helpers::option_error;
use crate::translators::tokens::Tokens;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorLanguages, TranslatorNoContext,
};

pub struct DeeplTranslator {
    /// auth key
    auth: String,
    /// host url
    host: String,
}

#[async_trait]
#[cfg(feature = "fetch_languages")]
impl TranslatorLanguages for DeeplTranslator {
    async fn get_languages(client: &Client, auth: &Tokens) -> Result<Vec<String>, Error> {
        let response = client
            .get("https://api-free.deepl.com/v2/languages?type=source")
            .header(
                "Authorization",
                format!(
                    "DeepL-Auth-Key {}",
                    auth.deepl_token
                        .as_ref()
                        .ok_or_else(|| Error::new_option("No deepl token"))?
                ),
            )
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|v| {
                Error::new(
                    "Failed get request from https://api-free.deepl.com/v2/languages?type=source",
                    v,
                )
            })?;
        if !response.status().is_success() {
            return Err(Error::new_option(format!(
                "Request failed with status code {}",
                response.status()
            )));
        }
        let json: Vec<DeeplLanguage> = response
            .json()
            .await
            .map_err(|v| Error::new("Failed to deserialize", v))?;
        Ok(json.iter().map(|v| v.code.to_string()).collect())
    }
}

#[async_trait]
impl TranslatorNoContext for DeeplTranslator {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        let response = self
            .request(
                client,
                query,
                option_error(from.map(|v| v.to_deepl_str()))?,
                &to.to_deepl_str()?,
            )
            .await?;
        let mut output = String::new();
        let mut language = String::new();
        for translation in response.translations {
            output.push_str(&translation.text);
            language = translation.detected_source_language;
        }
        Ok(TranslationOutput {
            text: output,
            lang: Language::from_str(&language)?,
        })
    }

    async fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let query = query.join("\n");
        let response = self
            .request(
                client,
                &query,
                option_error(from.map(|v| v.to_deepl_str()))?,
                &to.to_deepl_str()?,
            )
            .await?;
        let mut output: Vec<String> = Vec::new();
        let mut language = String::new();
        for translation in response.translations {
            output.push(translation.text.to_string());
            language = translation.detected_source_language;
        }
        Ok(TranslationVecOutput {
            text: output
                .join("\n")
                .split('\n')
                .map(|v| v.to_string())
                .collect(),
            lang: Language::from_str(&language)?,
        })
    }
}

impl DeeplTranslator {
    /// sets auth key and host url
    pub fn new(auth: &str) -> Self {
        DeeplTranslator {
            auth: auth.to_string(),
            host: "https://api-free.deepl.com/v2/translate".to_string(),
        }
    }

    /// Fetches the data and serializes it into a struct
    async fn request(
        &self,
        client: &Client,
        query: &str,
        from: Option<String>,
        target: &str,
    ) -> Result<TranslationResponse, Error> {
        let form = match from {
            Some(f) => vec![
                ("text", query.to_string()),
                ("target_lang", target.to_string()),
                ("source_lang", f),
            ],
            None => vec![
                ("text", query.to_string()),
                ("target_lang", target.to_string()),
            ],
        };
        let request = client
            .post(&self.host)
            .header("Authorization", format!("DeepL-Auth-Key {}", self.auth))
            .form(&form);
        let response = request
            .send()
            .await
            .map_err(|e| Error::new(format!("Failed post request to {}", self.host), e))?;
        if !response.status().is_success() {
            return Err(Error::new_option(format!(
                "Request failed with status code {}",
                response.status()
            )));
        }
        let json: TranslationResponse = response
            .json()
            .await
            .map_err(|e| Error::new("Failed to deserialize", e))?;
        Ok(json)
    }
}

/// Translation response of a single element
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Deserialize)]
pub struct Translation {
    /// input language
    pub detected_source_language: String,
    /// intput text
    pub text: String,
}

/// Translation response of multiple elements, containing a single element
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Deserialize)]
pub struct TranslationResponse {
    /// list of translations
    pub translations: Vec<Translation>,
}

/// Language response. Contains a single language. The result will be a vector of languages.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Deserialize)]
pub struct DeeplLanguage {
    /// identifier of language
    #[serde(alias = "language")]
    pub code: String,
    /// name of langauge
    pub name: String,
}

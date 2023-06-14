use std::str::FromStr;

use reqwest::blocking::Client;
use reqwest::header::{ORIGIN, REFERER};
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::languages::Language;
use crate::translators::helpers::option_error;
use crate::translators::tokens::Tokens;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorLanguages, TranslatorNoContext,
};

pub struct LibreTranslateTranslator {
    /// base url
    host: String,
    api_key: Option<String>,
}

impl TranslatorNoContext for LibreTranslateTranslator {
    fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        let data = LibreRequest {
            q: query.to_string(),
            source: option_error(from.map(|v| v.to_libretranslate_str()))?
                .unwrap_or_else(|| "auto".to_string()),
            target: to.to_libretranslate_str()?,
            format: String::from("text"),
            api_key: self.api_key.clone().unwrap_or_default(),
        };

        let req: TranslationResponses = client
            .post(format!("{}/translate", self.host))
            .header(REFERER, &self.host)
            .header(ORIGIN, &self.host)
            .json(&data)
            .send()
            .map_err(|e| Error::new(format!("Failed to send request to {}", self.host), e))?
            .json()
            .map_err(|e| Error::new("Failed to get response text", e))?;

        Ok(match req {
            TranslationResponses::WithDetectedLanguage(req) => TranslationOutput {
                text: req.translated_text,
                lang: Language::from_str(&req.detected_language.language)?,
            },
            TranslationResponses::WithoutDetectedLanguage(req) => TranslationOutput {
                text: req.translated_text,
                lang: Language::Unknown,
            },
        })
    }

    fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let v = self.translate(client, &query.join("\n"), from, to)?;
        Ok(TranslationVecOutput {
            text: v.text.split('\n').map(|v| v.to_string()).collect(),
            lang: v.lang,
        })
    }
}

#[cfg(feature = "fetch_languages")]
impl TranslatorLanguages for LibreTranslateTranslator {
    fn get_languages(client: &Client, _: &Tokens) -> Result<Vec<String>, Error> {
        let mut data: Vec<LanguagesResponse> = client
            .get("https://libretranslate.com/languages")
            .send()
            .map_err(|e| Error::new("Failed to get response text", e))?
            .json()
            .map_err(|e| Error::new("Failed to get response text", e))?;
        if data.is_empty() {
            return Err(Error::new(
                "Failed to get response text",
                "No data".to_string(),
            ));
        }
        Ok(data.remove(0).targets)
    }
}

impl LibreTranslateTranslator {
    pub fn new(api_key: &Option<String>) -> Self {
        //TODO: other urls
        //'https://translate.argosopentech.com', 'https://libretranslate.de', https://libretranslate.com,https://libretranslate.org,'https://trans.zillyhuhn.com',
        //'https://translate.astian.org', 'https://translate.mentality.rip','https://translate.api.skitzen.com',
        Self {
            host: "https://translate.argosopentech.com".to_string(),
            api_key: api_key.clone(),
        }
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LanguagesResponse {
    /// language code the targets belong to
    pub code: String,
    /// language name
    pub name: String,
    /// array of language codes that can be translated to
    pub targets: Vec<String>,
}

#[derive(Serialize)]
struct LibreRequest {
    /// text to translate
    q: String,
    /// source language
    /// default auto
    source: String,
    /// target language
    target: String,
    /// text format text
    format: String,
    /// api key(Optional)
    api_key: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DetectedLanguage {
    /// probability of the detected language
    pub confidence: f64,
    /// language code
    pub language: String,
}

/// Libretranslate
#[derive(Deserialize)]
struct TranslationResponse {
    /// Data of detected language
    #[serde(rename = "detectedLanguage")]
    pub detected_language: DetectedLanguage,
    /// translated text
    #[serde(rename = "translatedText")]
    pub translated_text: String,
}

/// Argos translate doenst have detected language
#[derive(Deserialize)]
struct TranslationResponse2 {
    /// translated text
    #[serde(rename = "translatedText")]
    pub translated_text: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TranslationResponses {
    /// Libretranslate
    WithDetectedLanguage(TranslationResponse),
    /// Argos
    WithoutDetectedLanguage(TranslationResponse2),
}

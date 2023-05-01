use std::fmt::{Debug, Formatter};
use async_trait::async_trait;
use reqwest::Client;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::context::Context;
use crate::translators::tokens::Tokens;

#[async_trait]
pub trait TranslatorNoContext {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error>;

    async fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error>;
}

#[async_trait]
pub trait TranslatorLanguages {
    async fn get_languages(client: &Client, auth: &Tokens) -> Result<Vec<String>, Error>;
}

#[async_trait]
pub trait DetectorApiBase {
    async fn get_language(client: &Client, query: &str, auth: &Tokens) -> Result<Language, Error>;
}

#[async_trait]
pub trait TranslatorContext {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
        context: &Vec<Context>,
    ) -> Result<TranslationOutput, Error>;

    async fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
        context: &Vec<Context>,
    ) -> Result<TranslationVecOutput, Error>;
}

pub enum TranslatorDyn {
    WC(Box<dyn TranslatorContext>),
    NC(Box<dyn TranslatorNoContext>),
}

impl Debug for TranslatorDyn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "No Debug")
    }
}

/// Translation Result containing the translation and the language
#[derive(Clone, Debug)]
pub struct TranslationOutput {
    /// Translation
    pub text: String,
    /// Text language
    pub lang: Language,
}

/// Translation Result containing a vector of translations and the language
#[derive(Clone, Debug)]
pub struct TranslationVecOutput {
    /// Translations
    pub text: Vec<String>,
    /// Language
    pub lang: Language,
}

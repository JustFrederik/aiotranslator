use std::future::Future;
#[cfg(feature = "retries")]
use std::time::Duration;
use std::vec;

#[cfg(feature = "retries")]
use futures::future::FutureExt;
#[cfg(not(feature = "offline_req"))]
use futures::{stream, StreamExt};
#[cfg(feature = "offline_req")]
use model_manager::model_manager::ModelManager;
use reqwest::Client;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use crate::detector::{detect_language, Detectors};
use crate::error::Error;
use crate::languages::Language;
use crate::translators::api::chatgpt::ChatGPTModel;
use crate::translators::chainer::{TranslatorSelectorInfo, TranslatorSelectorInitilized};
use crate::translators::context::Context;
#[cfg(feature = "offline_req")]
use crate::translators::offline::ctranslate2::model_management::{
    CTranslateModels, TokenizerModels,
};
#[cfg(feature = "offline_req")]
use crate::translators::offline::ctranslate2::Device;
#[cfg(feature = "jparacrawl")]
use crate::translators::offline::jparacrawl::JParaCrawlModelType;
#[cfg(feature = "m2m100")]
use crate::translators::offline::m2m100::M2M100ModelType;
#[cfg(feature = "nllb")]
use crate::translators::offline::nllb::NllbModelType;
#[cfg(feature = "offline_req")]
use crate::translators::offline::ModelFormat;
use crate::translators::tokens::Tokens;
use crate::translators::translator_initilized::TranslatorInitialized;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorDyn,
};

mod api;
pub mod chainer;
mod chatbot;
pub mod context;
pub mod dev;
mod helpers;
pub mod offline;
pub mod scrape;
pub mod tokens;
mod translator_initilized;
pub mod translator_structure;

#[derive(Default, PartialEq, Eq, EnumString, IntoStaticStr, Clone, Debug)]
pub enum ConversationStyleClone {
    Creative,
    #[default]
    Balanced,
    Precise,
}
/// Enum Containing all the translators
/// NOTE: when defining new translator add the is_api, is_fetch,get_api_available in the impl
#[derive(EnumIter, IntoStaticStr, Clone, PartialEq, Debug)]
pub enum Translator {
    /// For Deepl Translate with API key
    #[cfg(feature = "deepl")]
    Deepl,
    /// For Chatgpt Translate with API key structure Chatgpt(model, proxy_gpt3, proxy_gpt4, temperature)
    #[cfg(feature = "chatgpt")]
    ChatGPT(ChatGPTModel, String, String, f32),
    EdgeGPT(ConversationStyleClone, String),
    /// For Google Translate
    Google,
    /// For Bing Translate
    #[cfg(feature = "bing-scrape")]
    Bing,
    /// For Libre Translate
    #[cfg(feature = "libre")]
    LibreTranslate,
    /// For Mymemory Translate
    #[cfg(feature = "mymemory")]
    MyMemory,
    /// For Papago Translate
    #[cfg(feature = "papago-scrape")]
    Papago,
    /// For Youdao Translate
    #[cfg(feature = "youdao-scrape")]
    Youdao,
    /// For Baidu Translate
    #[cfg(feature = "baidu-scrape")]
    Baidu,
    #[cfg(feature = "nllb")]
    Nllb(Device, ModelFormat, NllbModelType),
    #[cfg(feature = "m2m100")]
    M2M100(Device, ModelFormat, M2M100ModelType),
    #[cfg(feature = "jparacrawl")]
    JParaCrawl(Device, ModelFormat, JParaCrawlModelType),
    #[cfg(feature = "sugoi")]
    Sugoi(Device, ModelFormat),
}

impl Translator {
    /// Get all translators
    pub fn get_all() -> Vec<Self> {
        Self::iter().collect()
    }

    /// gets strings
    pub fn convert_to_str(v: Vec<Translator>) -> Vec<&'static str> {
        v.into_iter().map(|v| v.into()).collect()
    }

    /// Get api translators
    pub fn get_online() -> Vec<Self> {
        Self::iter().filter(|v| v.is_online()).collect()
    }

    /// Get offline translators
    pub fn get_offline() -> Vec<Self> {
        Self::iter().filter(|v| !v.is_online()).collect()
    }

    /// Returns all apis that fetch from public translate
    pub fn get_scraped() -> Vec<Self> {
        Self::iter().filter(|v| v.is_scraped()).collect()
    }

    /// Return all the api translators with token
    pub fn get_api() -> Vec<Self> {
        Self::iter().filter(|v| v.is_api()).collect()
    }

    /// Return all the api translators with token, but only when token is available
    pub fn get_api_available(tokens: &Tokens) -> Vec<Self> {
        Self::iter()
            .filter(|v| v.is_api())
            .filter(|v| match v {
                Translator::Deepl => tokens.deepl_token.is_some(),
                Translator::ChatGPT(_, _, _, _) => tokens.gpt_token.is_some(),
                Translator::LibreTranslate => tokens.libre_token.is_some(),
                _ => false,
            })
            .collect()
    }

    /// Returns true when using an API with token
    fn is_api(&self) -> bool {
        matches!(
            self,
            Translator::Deepl
                | Translator::ChatGPT(_, _, _, _)
                | Translator::LibreTranslate
                | Translator::MyMemory
        )
    }

    /// Returns true when scraping translation from public translate
    fn is_scraped(&self) -> bool {
        matches!(
            self,
            Translator::Google
                | Translator::Bing
                | Translator::Papago
                | Translator::Youdao
                | Translator::Baidu
        )
    }

    /// Returns true if the translator is api
    fn is_online(&self) -> bool {
        self.is_scraped() || self.is_api()
    }
}

/// The Translation Service Struct
pub struct Translators {
    /// Translators that are used, like chain, selctive or selective chain
    pub translators: TranslatorSelectorInitilized,
    /// The delay between retries, when a translator fails
    #[cfg(feature = "retries")]
    pub retry_delay: Option<Duration>,
    /// The number of retries, when a translator fails, if None, it will retry forever
    #[cfg(feature = "retries")]
    pub retry_count: Option<u32>,
    /// Choose between langauge detection
    pub detector: Detectors,
    /// Struct with all tokens
    pub tokens: Tokens,
    /// Reqwest client. This is used for all requests except for the chatgpt translator
    pub client: Client,
    pub max_sim_conn: usize,
}

impl Translators {
    /// returns the default translator that will be used when no translator for the specific case is specified
    fn get_default_translator(&self) -> Result<&TranslatorInitialized, Error> {
        match &self.translators {
            TranslatorSelectorInitilized::SelectiveChain(v) => v
                .get(&Language::Unknown)
                .ok_or_else(|| Error::new_option("No default value set")),
            TranslatorSelectorInitilized::Selective(v) => v
                .get(&Language::Unknown)
                .ok_or_else(|| Error::new_option("No default value set")),
            _ => Err(Error::new_option("No default")),
        }
    }
    /// This will initiliaze the translator Service
    pub async fn new(
        tokens: Option<Tokens>,
        selector: TranslatorSelectorInfo,
        #[cfg(feature = "retries")] retry_delay: Option<Duration>,
        #[cfg(feature = "retries")] retry_count: Option<u32>,
        detector: Detectors,
        #[cfg(feature = "offline_req")] model_manager: &ModelManager,
    ) -> Result<Self, Error> {
        let client = Default::default();
        let tokens = match tokens {
            Some(v) => Ok(v),
            None => Tokens::get_env(),
        }
        .map_err(|v| Error::new("Couldnt get tokens from env", v))?;
        let translators = Self::generate_chain(
            selector,
            &tokens,
            5,
            &client,
            #[cfg(feature = "offline_req")]
            model_manager,
        )
        .await?;
        Ok(Self {
            translators,
            #[cfg(feature = "retries")]
            retry_delay,
            #[cfg(feature = "retries")]
            retry_count,
            detector,
            tokens,
            client,
            max_sim_conn: 5,
        })
    }

    /// This will generate the chain of translators/selective translators and the default translator
    async fn generate_chain(
        selector: TranslatorSelectorInfo,
        tokens: &Tokens,
        cc: usize,
        client: &Client,
        #[cfg(feature = "offline_req")] model_manager: &ModelManager,
    ) -> Result<TranslatorSelectorInitilized, Error> {
        let check_available = |lang: &Language, translator: &Translator| -> Result<String, Error> {
            match translator {
                Translator::Deepl => lang.to_deepl_str(),
                Translator::ChatGPT(_, _, _, _) | Translator::EdgeGPT(_, _) => lang.to_name_str(),
                Translator::Google => lang.to_google_str(),
                Translator::Bing => lang.to_bing_str(),
                Translator::LibreTranslate => lang.to_libretranslate_str(),
                Translator::MyMemory => lang.to_mymemory_str(),
                Translator::Papago => lang.to_papago_str(),
                Translator::Youdao => lang.to_youdao_str(),
                Translator::Baidu => lang.to_baidu_str(),
                #[cfg(feature = "nllb")]
                Translator::Nllb(_, _, _) => lang.to_nllb_str(),
                #[cfg(feature = "m2m100")]
                Translator::M2M100(_, _, _) => lang.to_m2m100_str(),
                #[cfg(feature = "jparacrawl")]
                Translator::JParaCrawl(_, _, _) => lang.to_jparacrawl_str(),
                #[cfg(feature = "sugoi")]
                Translator::Sugoi(_, _) => lang.to_sugoi_str(),
            }
        };
        match &selector {
            TranslatorSelectorInfo::Selective(g, def) => {
                for value in g {
                    check_available(&def.to, &def.translator)?;
                    check_available(value.0, value.1)?;
                }
            }
            TranslatorSelectorInfo::SelectiveChain(g, def) => {
                check_available(&def.to, &def.translator)?;
                for value in g {
                    check_available(value.0, &value.1.translator)?;
                    check_available(&value.1.to, &value.1.translator)?;
                }
            }
            TranslatorSelectorInfo::Chain(g) => {
                for value in g {
                    check_available(&value.to, &value.translator)?;
                }
            }
            TranslatorSelectorInfo::List(g) => {
                for value in g {
                    check_available(&value.to, &value.translator)?;
                }
            }
        };
        TranslatorSelectorInitilized::from_info(
            selector,
            tokens,
            cc,
            client,
            #[cfg(feature = "offline_req")]
            model_manager,
        )
        .await
    }

    /// The call to translate a string
    pub async fn translate(
        &self,
        text: String,
        from: Option<Language>,
        context_data: &[Context],
        #[cfg(feature = "offline_req")] translator_models: &mut CTranslateModels,
        #[cfg(feature = "offline_req")] tokenizer_models: &mut TokenizerModels,
    ) -> Result<Vec<TranslationOutput>, Error> {
        let add_from_lang = from.is_some();
        let lang = self.get_lang(from, &text)?;
        let from = match add_from_lang {
            true => Some(lang),
            false => None,
        };
        let chain = self.get_translator_chain(&lang)?;
        if chain.is_empty() {
            return Err(Error::new_option("No translator found"));
        }

        let mut translations: Vec<TranslationOutput> = vec![TranslationOutput {
            text: text.to_string(),
            lang,
        }];

        match &self.translators {
            TranslatorSelectorInitilized::List(items) => {
                let (queries, from) = match Self::need_lang(items) {
                    true => {
                        let mut v = translations.first_mut().ok_or_else(|| {
                            Error::new_option("initial translation value not set")
                        })?;
                        if v.lang == Language::Unknown {
                            v.lang = detect_language(&v.text, &self.detector)?;
                        }
                        (v.text.to_string(), Some(v.lang))
                    }
                    false => (
                        translations
                            .first()
                            .ok_or_else(|| Error::new_option("initial translation value not set"))?
                            .text
                            .to_string(),
                        from,
                    ),
                };

                //TODO: replace with multithreaded version
                #[cfg(not(feature = "offline_req"))]
                let u = stream::iter(items)
                    .map(|v| async { self.translate_fetch(&queries, from, context_data, v).await })
                    .buffer_unordered(self.max_sim_conn);
                #[cfg(not(feature = "offline_req"))]
                let v = u.collect::<Vec<Result<TranslationOutput, Error>>>().await;
                #[cfg(feature = "offline_req")]
                let mut v = vec![];
                #[cfg(feature = "offline_req")]
                for item in items {
                    v.push(
                        self.translate_fetch(
                            &queries,
                            from,
                            context_data,
                            item,
                            translator_models,
                            tokenizer_models,
                        )
                        .await,
                    );
                }
                for value in v {
                    translations.push(value?);
                }
            }
            _ => {
                for translator in chain {
                    let (query, from) = match &translator.translator {
                        Translator::MyMemory => {
                            let mut v = translations.last_mut().ok_or_else(|| {
                                Error::new_option("initial translation value not set")
                            })?;
                            if v.lang == Language::Unknown {
                                v.lang = detect_language(&v.text, &self.detector)?;
                            }
                            (v.text.to_string(), Some(v.lang))
                        }
                        _ => (
                            translations
                                .last()
                                .ok_or_else(|| Error::new_option("No translation value set"))?
                                .text
                                .to_string(),
                            from,
                        ),
                    };
                    #[cfg(feature = "offline_req")]
                    let text = self
                        .translate_fetch(
                            &query,
                            from,
                            context_data,
                            translator,
                            translator_models,
                            tokenizer_models,
                        )
                        .await?;
                    #[cfg(not(feature = "offline_req"))]
                    let text = self
                        .translate_fetch(&query, from, context_data, translator)
                        .await?;
                    if translations.len() == 1 {
                        if let Some(v) = translations.last_mut() {
                            if v.lang == Language::Unknown && text.lang != Language::Unknown {
                                v.lang = text.lang;
                            }
                        }
                    }

                    translations.push(text);
                }
            }
        }

        Ok(translations)
    }

    async fn translate_fetch(
        &self,
        query: &str,
        from: Option<Language>,
        context_data: &[Context],
        translator: &TranslatorInitialized,
        #[cfg(feature = "offline_req")] translator_models: &mut CTranslateModels,
        #[cfg(feature = "offline_req")] tokenizer_models: &mut TokenizerModels,
    ) -> Result<TranslationOutput, Error> {
        let text = match &translator.data {
            TranslatorDyn::WC(v) => {
                self.retry(v.translate(&self.client, query, from, &translator.to, context_data))
                    .await?
            }
            TranslatorDyn::NC(v) => {
                self.retry(v.translate(&self.client, query, from, &translator.to))
                    .await?
            }
            #[cfg(feature = "offline_req")]
            TranslatorDyn::Of(v) => v.translate(
                translator_models,
                tokenizer_models,
                query,
                from,
                &translator.to,
            )?,
        };

        Ok(TranslationOutput {
            text: text.text,
            lang: translator.to,
        })
    }

    /// The call to translate a vec of strings
    pub async fn translate_vec(
        &self,
        queries: Vec<String>,
        from: Option<Language>,
        context_data: &[Context],
        #[cfg(feature = "offline_req")] translator_models: &mut CTranslateModels,
        #[cfg(feature = "offline_req")] tokenizer_models: &mut TokenizerModels,
    ) -> Result<Vec<TranslationVecOutput>, Error> {
        let add_from_lang = from.is_some();
        let lang = self.get_lang(from, &queries.join("\n"))?;
        let from = match add_from_lang {
            true => Some(lang),
            false => None,
        };
        let chain = self.get_translator_chain(&lang)?;
        if chain.is_empty() {
            return Err(Error::new_option("No translator found"));
        }

        let mut translations: Vec<TranslationVecOutput> = vec![TranslationVecOutput {
            text: queries,
            lang,
        }];

        match &self.translators {
            TranslatorSelectorInitilized::List(items) => {
                let (queries, from) = match Self::need_lang(items) {
                    true => {
                        let mut v = translations.first_mut().ok_or_else(|| {
                            Error::new_option("initial translation value not set")
                        })?;
                        if v.lang == Language::Unknown {
                            v.lang = detect_language(&v.text.join("\n"), &self.detector)?;
                        }
                        (&v.text, Some(v.lang))
                    }
                    false => (
                        &translations
                            .first()
                            .ok_or_else(|| Error::new_option("initial translation value not set"))?
                            .text,
                        from,
                    ),
                };
                //TODO: replace with multithreaded version
                #[cfg(not(feature = "offline_req"))]
                let u = stream::iter(items)
                    .map(|v| async {
                        self.translate_vec_fetch(queries, from, v, context_data)
                            .await
                    })
                    .buffer_unordered(self.max_sim_conn);
                #[cfg(not(feature = "offline_req"))]
                let v = u
                    .collect::<Vec<Result<TranslationVecOutput, Error>>>()
                    .await;
                #[cfg(feature = "offline_req")]
                let mut v = vec![];
                #[cfg(feature = "offline_req")]
                for item in items {
                    v.push(
                        self.translate_vec_fetch(
                            queries,
                            from,
                            item,
                            context_data,
                            translator_models,
                            tokenizer_models,
                        )
                        .await,
                    );
                }
                for value in v {
                    translations.push(value?);
                }
            }
            _ => {
                for translator in chain {
                    let (queries, from) = match &translator.translator {
                        Translator::MyMemory => {
                            let mut v = translations.last_mut().ok_or_else(|| {
                                Error::new_option("initial translation value not set")
                            })?;
                            if v.lang == Language::Unknown {
                                v.lang = detect_language(&v.text.join("\n"), &self.detector)?;
                            }
                            (&v.text, Some(v.lang))
                        }
                        _ => (
                            &translations
                                .last()
                                .ok_or_else(|| {
                                    Error::new_option("initial translation value not set")
                                })?
                                .text,
                            from,
                        ),
                    };
                    #[cfg(feature = "offline_req")]
                    let text = self
                        .translate_vec_fetch(
                            queries,
                            from,
                            translator,
                            context_data,
                            translator_models,
                            tokenizer_models,
                        )
                        .await?;
                    #[cfg(not(feature = "offline_req"))]
                    let text = self
                        .translate_vec_fetch(queries, from, translator, context_data)
                        .await?;
                    if translations.len() == 1 {
                        if let Some(v) = translations.last_mut() {
                            if v.lang == Language::Unknown && text.lang != Language::Unknown {
                                v.lang = text.lang;
                            }
                        }
                    }
                    translations.push(text);
                }
            }
        }

        Ok(translations)
    }

    async fn translate_vec_fetch(
        &self,
        queries: &[String],
        from: Option<Language>,
        translator: &TranslatorInitialized,
        context_data: &[Context],
        #[cfg(feature = "offline_req")] translator_models: &mut CTranslateModels,
        #[cfg(feature = "offline_req")] tokenizer_models: &mut TokenizerModels,
    ) -> Result<TranslationVecOutput, Error> {
        match &translator.data {
            TranslatorDyn::WC(v) => {
                self.retry(v.translate_vec(
                    &self.client,
                    queries,
                    from,
                    &translator.to,
                    context_data,
                ))
                .await
            }
            TranslatorDyn::NC(v) => {
                self.retry(v.translate_vec(&self.client, queries, from, &translator.to))
                    .await
            }
            #[cfg(feature = "offline_req")]
            TranslatorDyn::Of(v) => v.translate_vec(
                translator_models,
                tokenizer_models,
                queries,
                from,
                &translator.to,
            ),
        }
    }

    /// Function to retry on error
    #[cfg(feature = "retries")]
    async fn retry<F: Future<Output = Result<T, Error>>, T: Clone>(
        &self,
        f: F,
    ) -> Result<T, Error> {
        let mut retry_count = 0;
        let max = self.retry_count.unwrap_or(1);
        let delay = self.retry_delay.unwrap_or(Duration::from_secs(1));
        let mut res = Err(Error::new_option("No result yet"));
        let f = f.shared();
        while retry_count < max {
            let fc = f.clone();
            res = fc.await;
            if res.is_ok() {
                break;
            }
            if self.retry_count.is_some() {
                retry_count += 1;
            }
            tokio::time::sleep(delay).await;
        }
        res
    }

    #[cfg(not(feature = "retries"))]
    /// Function that does nothing, but is needed for retries feature.
    /// This function will be called when the retry feature is not enabled.
    async fn retry<F: Future<Output = Result<T, Error>>, T: Clone>(
        &self,
        f: F,
    ) -> Result<T, Error> {
        f.await
    }

    /// This generates a chain of translators. If no translator is found, it will add the default translator as the last translator.
    fn get_translator_chain(
        &self,
        lang_iso: &Language,
    ) -> Result<Vec<&TranslatorInitialized>, Error> {
        let mut res = vec![];
        match &self.translators {
            TranslatorSelectorInitilized::Chain(v) => v.iter().for_each(|v| res.push(v)),
            TranslatorSelectorInitilized::List(v) => v.iter().for_each(|v| res.push(v)),
            TranslatorSelectorInitilized::Selective(v) => match v.get(lang_iso) {
                Some(v) => res.push(v),
                None => res.push(self.get_default_translator()?),
            },
            TranslatorSelectorInitilized::SelectiveChain(v) => {
                let mut li = lang_iso;
                let default = self.get_default_translator()?;
                loop {
                    let trans = match v.get(li) {
                        Some(v) => Ok(v),
                        None => Err("No translator for language".to_string()),
                    };
                    if let Ok(t) = trans {
                        res.push(t);
                        li = &t.to;
                        if t.to == default.to {
                            break;
                        }
                        continue;
                    }
                    res.push(default);
                    break;
                }
            }
        }
        Ok(res)
    }

    fn get_lang(&self, from: Option<Language>, text: &str) -> Result<Language, Error> {
        match from {
            Some(v) => Ok(v),
            None => match self.translators {
                TranslatorSelectorInitilized::Chain(_) | TranslatorSelectorInitilized::List(_) => {
                    Ok(Language::Unknown)
                }
                _ => detect_language(text, &self.detector),
            },
        }
    }

    fn need_lang(items: &Vec<TranslatorInitialized>) -> bool {
        let mut need_lang = false;
        for v in items {
            if v.translator == Translator::MyMemory {
                need_lang = true;
            }
        }
        need_lang
    }
}

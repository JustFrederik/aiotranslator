use std::fmt::Formatter;
use std::str::FromStr;
use std::time::Duration;
use std::vec;

use log::info;
#[cfg(feature = "ctranslate_req")]
use model_manager::model_manager::ModelManager;
use reqwest::blocking::Client;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::detector::{detect_language, Detectors};
use crate::error::Error;
use crate::languages::Language;
use crate::translators::api::chatgpt::ChatGPTModel;
use crate::translators::chainer::{TranslatorSelectorInfo, TranslatorSelectorInitilized};
use crate::translators::context::Context;
#[cfg(feature = "ctranslate_req")]
use crate::translators::offline::ctranslate2::model_management::{
    CTranslateModels, TokenizerModels,
};
#[cfg(feature = "ctranslate_req")]
use crate::translators::offline::ctranslate2::Device;
#[cfg(feature = "jparacrawl")]
use crate::translators::offline::jparacrawl::JParaCrawlModelType;
#[cfg(feature = "m2m100")]
use crate::translators::offline::m2m100::M2M100ModelType;
#[cfg(feature = "nllb")]
use crate::translators::offline::nllb::NllbModelType;
#[cfg(feature = "ctranslate_req")]
use crate::translators::offline::ModelFormat;
use crate::translators::tokens::Tokens;
use crate::translators::translator_initilized::TranslatorInitialized;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorDyn,
};

pub mod api;
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

#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub enum ConversationStyleClone {
    Creative,
    #[default]
    Balanced,
    Precise,
}

impl FromStr for ConversationStyleClone {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Creative" => Ok(Self::Creative),
            "Balanced" => Ok(Self::Balanced),
            "Precise" => Ok(Self::Precise),
            _ => Err(()),
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum TranslatorKind {
    #[default]
    Api,
    #[cfg(any(feature = "youdao-scrape", feature = "baidu-scrape"))]
    Scrape,
}

impl TranslatorKind {
    pub fn new_api(y: bool) -> Self {
        if y {
            Self::Api
        } else {
            Self::Scrape
        }
    }
}

/// Enum Containing all the translators
/// NOTE: when defining new translator add the is_api, is_fetch,get_api_available in the impl
#[derive(EnumIter, Clone, PartialEq, Debug)]
pub enum Translator {
    /// For Deepl Translate with API key
    #[cfg(feature = "deepl")]
    Deepl,
    /// For Chatgpt Translate with API key structure Chatgpt(model, proxy_gpt3, proxy_gpt4, temperature)
    #[cfg(feature = "chatgpt")]
    ChatGPT(ChatGPTModel, String, String, f32, Duration),
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
    #[cfg(any(feature = "youdao-scrape", feature = "youdao"))]
    Youdao(TranslatorKind),
    /// For Baidu Translate
    #[cfg(any(feature = "baidu-scrape", feature = "baidu"))]
    Baidu(TranslatorKind),
    #[cfg(feature = "nllb")]
    Nllb(Device, ModelFormat, NllbModelType),
    #[cfg(feature = "m2m100")]
    M2M100(Device, ModelFormat, M2M100ModelType),
    #[cfg(feature = "jparacrawl")]
    JParaCrawl(Device, ModelFormat, JParaCrawlModelType),
    #[cfg(feature = "sugoi")]
    Sugoi(Device, ModelFormat),
}

impl std::fmt::Display for Translator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "deepl")]
            Translator::Deepl => write!(f, "Deepl"),
            #[cfg(feature = "chatgpt")]
            Translator::ChatGPT(_, _, _, _, _) => write!(f, "ChatGPT"),
            #[cfg(feature = "edge-gpt-scrape")]
            Translator::EdgeGPT(_, _) => write!(f, "EdgeGPT"),
            #[cfg(feature = "google-scrape")]
            Translator::Google => write!(f, "Google"),
            #[cfg(feature = "bing-scrape")]
            Translator::Bing => write!(f, "Bing"),
            #[cfg(feature = "libre")]
            Translator::LibreTranslate => write!(f, "LibreTranslate"),
            #[cfg(feature = "mymemory")]
            Translator::MyMemory => write!(f, "MyMemory"),
            #[cfg(feature = "papago")]
            Translator::Papago => write!(f, "Papago"),
            #[cfg(feature = "youdao")]
            Translator::Youdao(_) => write!(f, "Youdao"),
            #[cfg(feature = "baidu")]
            Translator::Baidu(_) => write!(f, "Baidu"),
            #[cfg(feature = "nllb")]
            Translator::Nllb(_, _, _) => write!(f, "Nllb"),
            #[cfg(feature = "m2m100")]
            Translator::M2M100(_, _, _) => write!(f, "M2M100"),
            #[cfg(feature = "jparacrawl")]
            Translator::JParaCrawl(_, _, _) => write!(f, "JparaCrawl"),
            #[cfg(feature = "sugoi")]
            Translator::Sugoi(_, _) => write!(f, "Sugui"),
        }
    }
}

impl FromStr for Translator {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = match s.contains(' ') {
            true => s.split(' ').collect::<Vec<&str>>(),
            false => vec![s],
        };
        Ok(match s.remove(0) {
            #[cfg(feature = "deepl")]
            "deepl" => Self::Deepl,
            #[cfg(feature = "chatgpt")]
            "chatgpt" => {
                let model = ChatGPTModel::from_str(s.remove(0)).map_err(|_| ())?;
                let temp = s.remove(0).parse().map_err(|_| ())?;
                let wait_ms = s.remove(0).parse().map_err(|_| ())?;
                Self::ChatGPT(
                    model,
                    "".to_string(),
                    "".to_string(),
                    temp,
                    Duration::from_millis(wait_ms),
                )
            }
            "edgegpt" => {
                let style = ConversationStyleClone::from_str(s.remove(0)).map_err(|_| ())?;
                let auth = s.remove(0);
                Self::EdgeGPT(style, auth.to_string())
            }
            "google" => Self::Google,
            #[cfg(feature = "bing-scrape")]
            "bing" => Self::Bing,
            #[cfg(feature = "libre")]
            "libretranslate" => Self::LibreTranslate,
            #[cfg(feature = "mymemory")]
            "mymemory" => Self::MyMemory,
            #[cfg(feature = "papago-scrape")]
            "papago" => Self::Papago,
            #[cfg(any(feature = "youdao-scrape", feature = "youdao"))]
            "youdao" => {
                let tk = TranslatorKind::new_api(s.remove(0).parse().map_err(|_| ())?);
                Self::Youdao(tk)
            }
            #[cfg(any(feature = "baidu-scrape", feature = "baidu"))]
            "baidu" => {
                let tk = TranslatorKind::new_api(s.remove(0).parse().map_err(|_| ())?);
                Self::Baidu(tk)
            }
            #[cfg(feature = "nllb")]
            "nllb" => {
                let d = Device::gpu(s.remove(0).parse().map_err(|_| ())?);
                let mf = ModelFormat::Compact;
                let mtype = NllbModelType::from_str(s.remove(0)).map_err(|_| ())?;
                Self::Nllb(d, mf, mtype)
            }
            #[cfg(feature = "m2m100")]
            "m2m100" => {
                let d = Device::gpu(s.remove(0).parse().map_err(|_| ())?);
                let mtype = M2M100ModelType::from_str(s.remove(0)).map_err(|_| ())?;
                Self::M2M100(d, ModelFormat::Compact, mtype)
            }
            #[cfg(feature = "jparacrawl")]
            "jparacrawl" => {
                let d = Device::gpu(s.remove(0).parse().map_err(|_| ())?);
                let mtype = JParaCrawlModelType::from_str(s.remove(0)).map_err(|_| ())?;
                Self::JParaCrawl(d, ModelFormat::Compact, mtype)
            }
            #[cfg(feature = "sugoi")]
            "sugoi" => {
                let d = Device::gpu(s.remove(0).parse().map_err(|_| ())?);
                Self::Sugoi(d, ModelFormat::Compact)
            }
            _ => return Err(()),
        })
    }
}

impl Translator {
    /// Get all translators
    pub fn get_all() -> Vec<Self> {
        Self::iter().collect()
    }

    /// gets strings
    pub fn convert_to_str(v: Vec<Translator>) -> Vec<String> {
        v.into_iter().map(|v| v.to_string()).collect()
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
                Translator::ChatGPT(_, _, _, _, _) => tokens.gpt_token.is_some(),
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
                | Translator::ChatGPT(_, _, _, _, _)
                | Translator::LibreTranslate
                | Translator::MyMemory
                | Translator::Youdao(TranslatorKind::Api)
                | Translator::Baidu(TranslatorKind::Api)
        )
    }

    /// Returns true when scraping translation from public translate
    fn is_scraped(&self) -> bool {
        matches!(
            self,
            Translator::Google
                | Translator::Bing
                | Translator::Papago
                | Translator::Youdao(TranslatorKind::Scrape)
                | Translator::Baidu(TranslatorKind::Scrape)
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

    pub retry_delay: Option<Duration>,

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
    pub fn new(
        tokens: Option<Tokens>,
        selector: TranslatorSelectorInfo,
        retry_delay: Option<Duration>,
        retry_count: Option<u32>,
        detector: Detectors,
        #[cfg(feature = "ctranslate_req")] model_manager: &ModelManager,
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
            &client,
            #[cfg(feature = "ctranslate_req")]
            model_manager,
        )?;
        Ok(Self {
            translators,
            retry_delay,
            retry_count,
            detector,
            tokens,
            client,
            max_sim_conn: 5,
        })
    }

    /// This will generate the chain of translators/selective translators and the default translator
    fn generate_chain(
        selector: TranslatorSelectorInfo,
        tokens: &Tokens,
        client: &Client,
        #[cfg(feature = "ctranslate_req")] model_manager: &ModelManager,
    ) -> Result<TranslatorSelectorInitilized, Error> {
        let check_available = |lang: &Language, translator: &Translator| -> Result<String, Error> {
            match translator {
                Translator::Deepl => lang.to_deepl_str(),
                Translator::ChatGPT(_, _, _, _, _) | Translator::EdgeGPT(_, _) => {
                    lang.to_name_str()
                }
                Translator::Google => lang.to_google_str(),
                Translator::Bing => lang.to_bing_str(),
                Translator::LibreTranslate => lang.to_libretranslate_str(),
                Translator::MyMemory => lang.to_mymemory_str(),
                Translator::Papago => lang.to_papago_str(),
                Translator::Youdao(_) => lang.to_youdao_str(),
                Translator::Baidu(_) => lang.to_baidu_str(),
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
            client,
            #[cfg(feature = "ctranslate_req")]
            model_manager,
        )
    }

    /// The call to translate a string
    pub fn translate(
        &self,
        text: String,
        from: Option<Language>,
        context_data: &[Context],
        #[cfg(feature = "ctranslate_req")] translator_models: &mut CTranslateModels,
        #[cfg(feature = "ctranslate_req")] tokenizer_models: &mut TokenizerModels,
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

        let mut translations: Vec<TranslationOutput> = vec![TranslationOutput { text, lang }];

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
                let mut v = vec![];
                for item in items {
                    v.push(self.translate_fetch(
                        &queries,
                        from,
                        context_data,
                        item,
                        #[cfg(feature = "ctranslate_req")]
                        translator_models,
                        #[cfg(feature = "ctranslate_req")]
                        tokenizer_models,
                    ));
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
                    #[cfg(feature = "ctranslate_req")]
                    let text = self.translate_fetch(
                        &query,
                        from,
                        context_data,
                        translator,
                        translator_models,
                        tokenizer_models,
                    )?;

                    #[cfg(not(feature = "ctranslate_req"))]
                    let text = self.translate_fetch(&query, from, context_data, translator)?;
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

    fn translate_fetch(
        &self,
        query: &str,
        from: Option<Language>,
        context_data: &[Context],
        translator: &TranslatorInitialized,
        #[cfg(feature = "ctranslate_req")] translator_models: &mut CTranslateModels,
        #[cfg(feature = "ctranslate_req")] tokenizer_models: &mut TokenizerModels,
    ) -> Result<TranslationOutput, Error> {
        info!(
            "Translate \"{}\" with {}",
            query,
            translator.translator.to_string()
        );
        let text = match &translator.data {
            TranslatorDyn::WC(v) => {
                let mut temp;
                let mut retry = 0;
                loop {
                    temp = v.translate(&self.client, query, from, &translator.to, context_data);
                    retry += 1;
                    if temp.is_ok() || retry > self.retry_count.unwrap_or(3) {
                        break;
                    }
                }
                temp?
            }
            TranslatorDyn::NC(v) => {
                let mut temp;
                let mut retry = 0;
                loop {
                    temp = v.translate(&self.client, query, from, &translator.to);
                    retry += 1;
                    if temp.is_ok() || retry > self.retry_count.unwrap_or(3) {
                        break;
                    }
                }
                temp?
            }
            #[cfg(feature = "ctranslate_req")]
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
    pub fn translate_vec(
        &self,
        queries: Vec<String>,
        from: Option<Language>,
        context_data: &[Context],
        #[cfg(feature = "ctranslate_req")] translator_models: &mut CTranslateModels,
        #[cfg(feature = "ctranslate_req")] tokenizer_models: &mut TokenizerModels,
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
                let mut v = vec![];
                for item in items {
                    v.push(self.translate_vec_fetch(
                        queries,
                        from,
                        item,
                        context_data,
                        #[cfg(feature = "ctranslate_req")]
                        translator_models,
                        #[cfg(feature = "ctranslate_req")]
                        tokenizer_models,
                    ));
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
                    let text = self.translate_vec_fetch(
                        queries,
                        from,
                        translator,
                        context_data,
                        #[cfg(feature = "ctranslate_req")]
                        translator_models,
                        #[cfg(feature = "ctranslate_req")]
                        tokenizer_models,
                    )?;
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

    fn translate_vec_fetch(
        &self,
        queries: &[String],
        from: Option<Language>,
        translator: &TranslatorInitialized,
        context_data: &[Context],
        #[cfg(feature = "ctranslate_req")] translator_models: &mut CTranslateModels,
        #[cfg(feature = "ctranslate_req")] tokenizer_models: &mut TokenizerModels,
    ) -> Result<TranslationVecOutput, Error> {
        info!(
            "Translate {:?} with {}",
            queries,
            translator.translator.to_string()
        );
        match &translator.data {
            TranslatorDyn::WC(v) => {
                let mut temp;
                let mut retry = 0;
                loop {
                    temp =
                        v.translate_vec(&self.client, queries, from, &translator.to, context_data);
                    retry += 1;
                    if temp.is_ok() || retry > self.retry_count.unwrap_or(3) {
                        break;
                    }
                }
                temp
            }
            TranslatorDyn::NC(v) => {
                let mut temp;
                let mut retry = 0;
                loop {
                    temp = v.translate_vec(&self.client, queries, from, &translator.to);
                    retry += 1;
                    if temp.is_ok() || retry > self.retry_count.unwrap_or(3) {
                        break;
                    }
                }
                temp
            }
            #[cfg(feature = "ctranslate_req")]
            TranslatorDyn::Of(v) => v.translate_vec(
                translator_models,
                tokenizer_models,
                queries,
                from,
                &translator.to,
            ),
        }
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

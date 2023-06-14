use std::collections::HashMap;

#[cfg(feature = "ctranslate_req")]
use model_manager::model_manager::ModelManager;
use reqwest::blocking::Client;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::tokens::Tokens;
use crate::translators::translator_initilized::TranslatorInitialized;
use crate::translators::Translator;

/// An enum where it is defined if Selctive, SelectiveChain or Chain is used.
/// The Translators are not initialized yet.
#[derive(Clone)]
pub enum TranslatorSelectorInfo {
    /// A hashmap with the from langauge as a key and a TranslatorData as a value, which contains target langauge and the translator.
    /// Second value is default
    Selective(HashMap<Language, Translator>, TranslatorInfo),
    /// Equal to Selective, but the translator continues until the default translator is reached.
    /// Second value is default
    SelectiveChain(HashMap<Language, TranslatorInfo>, TranslatorInfo),
    /// Executes Translator after Translator until the end is reached
    Chain(Vec<TranslatorInfo>),
    List(Vec<TranslatorInfo>),
}

impl TranslatorSelectorInfo {
    pub fn create_single(v: TranslatorInfo) -> Self {
        Self::List(vec![v])
    }
}

/// An enum where it is defined if Selctive, SelectiveChain or Chain is used.
/// The translator is already initialized in the enum.
#[derive(Debug)]
pub enum TranslatorSelectorInitilized {
    /// A hashmap with the from langauge as a key and a TranslatorData as a value, which contains target langauge and the initialized translator.
    Selective(HashMap<Language, TranslatorInitialized>),
    /// Equal to Selective, but the translator continues until the default translator is reached.
    SelectiveChain(HashMap<Language, TranslatorInitialized>),
    /// Executes Translator after Translator until the end is reached
    Chain(Vec<TranslatorInitialized>),
    List(Vec<TranslatorInitialized>),
}

impl TranslatorSelectorInitilized {
    /// This function converts the TranslatorSelectorInfo to a TranslatorSelector and therefore initializes the translator.
    pub fn from_info(
        info: TranslatorSelectorInfo,
        tokens: &Tokens,
        client: &Client,
        #[cfg(feature = "ctranslate_req")] model_manager: &ModelManager,
    ) -> Result<Self, Error> {
        Ok(match info {
            TranslatorSelectorInfo::Selective(v, default) => {
                let mut v = v;
                v.insert(Language::Unknown, default.translator);
                Self::Selective(convert_selective_hashmap(
                    v,
                    tokens,
                    &default.to,
                    client,
                    #[cfg(feature = "ctranslate_req")]
                    model_manager,
                )?)
            }
            TranslatorSelectorInfo::SelectiveChain(v, default) => {
                let mut v = v;
                v.insert(Language::Unknown, default);
                Self::SelectiveChain(convert_selective_chain(
                    v,
                    tokens,
                    client,
                    #[cfg(feature = "ctranslate_req")]
                    model_manager,
                )?)
            }
            TranslatorSelectorInfo::Chain(v) => Self::Chain(convert_chain(
                v,
                tokens,
                client,
                #[cfg(feature = "ctranslate_req")]
                model_manager,
            )?),
            TranslatorSelectorInfo::List(v) => Self::List(convert_chain(
                v,
                tokens,
                client,
                #[cfg(feature = "ctranslate_req")]
                model_manager,
            )?),
        })
    }
}

/// Initializes every value in HashMap of Selective chain
fn convert_selective_chain(
    translator_info_map: HashMap<Language, TranslatorInfo>,
    tokens: &Tokens,
    client: &Client,
    #[cfg(feature = "ctranslate_req")] model_manager: &ModelManager,
) -> Result<HashMap<Language, TranslatorInitialized>, Error> {
    let mut translator_data_map = HashMap::new();

    for value in &translator_info_map {
        let mut keys = vec![value.0];
        let mut item = value.1;
        loop {
            let next = translator_info_map.get(&item.to);
            if let Some(next) = next {
                if keys.contains(&&item.to) {
                    return Err(Error::new_option("Translator loop detected"));
                }
                keys.push(&item.to);
                item = next;
            } else {
                break;
            }
        }
    }

    for (key, value) in translator_info_map {
        translator_data_map.insert(
            key,
            TranslatorInitialized::new(
                value,
                tokens,
                client,
                #[cfg(feature = "ctranslate_req")]
                model_manager,
            )?,
        );
    }

    Ok(translator_data_map)
}

/// Initializes every value in HashMap of Selective
fn convert_selective_hashmap(
    translator_info_map: HashMap<Language, Translator>,
    tokens: &Tokens,
    to: &Language,
    client: &Client,
    #[cfg(feature = "ctranslate_req")] model_manager: &ModelManager,
) -> Result<HashMap<Language, TranslatorInitialized>, Error> {
    let mut translator_data_map = HashMap::new();
    for (key, value) in translator_info_map {
        translator_data_map.insert(
            key,
            TranslatorInitialized::new(
                TranslatorInfo {
                    translator: value,
                    to: *to,
                },
                tokens,
                client,
                #[cfg(feature = "ctranslate_req")]
                model_manager,
            )?,
        );
    }
    Ok(translator_data_map)
}

/// Initializes every value in Vec of chain
fn convert_chain(
    vec: Vec<TranslatorInfo>,
    tokens: &Tokens,
    client: &Client,
    #[cfg(feature = "ctranslate_req")] model_manager: &ModelManager,
) -> Result<Vec<TranslatorInitialized>, Error> {
    let mut res = vec![];
    for v in vec {
        res.push(TranslatorInitialized::new(
            v,
            tokens,
            client,
            #[cfg(feature = "ctranslate_req")]
            model_manager,
        )?);
    }
    Ok(res)
}

#[derive(Clone)]
/// This represents a single translator with a target language
pub struct TranslatorInfo {
    /// Translator
    pub translator: Translator,
    /// Target language
    pub to: Language,
}

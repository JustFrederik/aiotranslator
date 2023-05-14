use std::collections::HashMap;
use std::path::{Path, PathBuf};

use model_manager::model_manager::ModelManager;
use rustyctranslate2::BatchType;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::offline::ctranslate2::model_management::{
    CTranslateModels, TokenizerModels,
};
use crate::translators::offline::ctranslate2::Device;
use crate::translators::offline::ModelFormat;
use crate::translators::translator_structure::{TranslationVecOutput, TranslatorCTranslate};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JParaCrawlModelType {
    Small,
    Base,
    #[default]
    Big,
}

pub struct JParaCrawlTranslator {
    device: Device,
    model_path: PathBuf,
    tokenizer_filenames: HashMap<Language, String>,
    ident: String,
    model_format: ModelFormat,
}

impl TranslatorCTranslate for JParaCrawlTranslator {
    fn translate_vec(
        &self,
        translator_models: &mut CTranslateModels,
        tokenizer_models: &mut TokenizerModels,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let from = Self::get_from(from, to)?;
        let tokenizer_path = self.model_path.join(
            self.tokenizer_filenames
                .get(&from)
                .ok_or_else(|| Error::new_option("Tokenizer not found"))?,
        );
        let translator_path = Self::get_translator_model_path(&self.model_path, from, to)?;
        let tokenizer = tokenizer_models.get_tokenizer(
            &format!("jparacrawl-{}", from.to_jparacrawl_str()?),
            tokenizer_path,
        )?;
        let tokens = tokenizer.tokenize(query)?;
        let translator = translator_models.get_translator(
            &format!(
                "{}-{}-{}",
                self.ident,
                from.to_jparacrawl_str()?,
                to.to_jparacrawl_str()?
            ),
            translator_path,
            &self.device,
            self.model_format.is_compressed(),
        )?;
        let translated = translator
            .translate_batch(tokens, None, BatchType::Example)
            .map_err(Error::new_option)?;
        let sentences = tokenizer.detokenize(translated)?;
        translator_models.cleanup();
        tokenizer_models.cleanup();
        Ok(TranslationVecOutput {
            text: sentences,
            lang: from,
        })
    }
}

impl JParaCrawlTranslator {
    pub async fn new(
        device: &Device,
        model_format: &ModelFormat,
        model_type: &JParaCrawlModelType,
        model_manager: &ModelManager,
    ) -> Result<Self, Error> {
        let ident = Self::get_ident(device, model_format, model_type);
        let model = model_manager
            .get_model_async(&ident)
            .await
            .map_err(|_| Error::new_option("couldnt get model".to_string()))?;
        Ok(Self {
            device: *device,
            model_path: model.0.join(&model.1.directory),
            tokenizer_filenames: Self::get_tokenizer_filenames(),
            ident,
            model_format: *model_format,
        })
    }

    pub fn get_translator_model_path(
        path: &Path,
        from: Language,
        to: &Language,
    ) -> Result<PathBuf, Error> {
        Ok(path.join(format!(
            "{}-{}",
            from.to_jparacrawl_str()?,
            to.to_jparacrawl_str()?
        )))
    }

    pub fn get_from(from: Option<Language>, to: &Language) -> Result<Language, Error> {
        if let Some(f) = from {
            if (f == Language::English && to == &Language::Japanese)
                || (f == Language::Japanese && to == &Language::English)
            {
                return Ok(f);
            }
            Err(Error::new_option("Language not supported"))?
        } else {
            Ok(match to {
                Language::English => Language::Japanese,
                Language::Japanese => Language::English,
                _ => Err(Error::new_option("Language not supported"))?,
            })
        }
    }

    pub fn get_tokenizer_filenames() -> HashMap<Language, String> {
        let mut tokenizer_filenames = HashMap::new();
        tokenizer_filenames.insert(Language::English, "spm.en.nopretok.model".to_string());
        tokenizer_filenames.insert(Language::Japanese, "spm.ja.nopretok.model".to_string());
        tokenizer_filenames
    }

    fn get_ident(
        device: &Device,
        model_format: &ModelFormat,
        model_type: &JParaCrawlModelType,
    ) -> String {
        let ending = match model_format {
            ModelFormat::Compact => match device {
                Device::CPU => "-int8",
                Device::CUDA => "-float16",
            },
            ModelFormat::Normal => "",
        };
        match model_type {
            JParaCrawlModelType::Small => format!("jparacrawl-small-ct2{}", ending),
            JParaCrawlModelType::Base => format!("jparacrawl-base-ct2{}", ending),
            JParaCrawlModelType::Big => format!("jparacrawl-big-ct2{}", ending),
        }
    }
}

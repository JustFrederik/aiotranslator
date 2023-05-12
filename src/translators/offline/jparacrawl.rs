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

pub enum JParaCrawlModelType {
    Small,
    Base,
    Big,
}

pub struct JParaCrawlTranslator<'a> {
    device: Device,
    model_manager: &'a ModelManager,
    tokenizer_filenames: HashMap<Language, String>,
    model_type: JParaCrawlModelType,
    model_format: ModelFormat,
}

impl<'a> TranslatorCTranslate for JParaCrawlTranslator<'a> {
    fn translate_vec(
        &self,
        translator_models: &mut CTranslateModels,
        tokenizer_models: &mut TokenizerModels,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let from = Self::get_from(from, to)?;
        let ident = self.get_ident();
        let model = self.model_manager.get_model(&ident).unwrap();
        let model_path = model.0.join(&model.1.directory);
        let tokenizer_path = model_path.join(
            self.tokenizer_filenames
                .get(&from)
                .ok_or_else(|| Error::new_option("Tokenizer not found"))?,
        );
        let translator_path = Self::get_translator_model_path(&model_path, from, to)?;
        let tokenizer = tokenizer_models.get_tokenizer(
            &format!("jparacrawl-{}", from.to_jparacrawl_str()?),
            tokenizer_path,
        )?;
        let tokens = tokenizer.tokenize(query)?;
        let translator = translator_models.get_translator(
            &format!(
                "{}-{}-{}",
                ident,
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

impl<'a> JParaCrawlTranslator<'a> {
    pub fn new(
        device: Device,
        model_type: JParaCrawlModelType,
        model_format: ModelFormat,
        model_manager: &'a ModelManager,
    ) -> Self {
        Self {
            device,
            model_manager,
            tokenizer_filenames: Self::get_tokenizer_filenames(),
            model_type,
            model_format,
        }
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

    fn get_ident(&self) -> String {
        let ending = match self.model_format {
            ModelFormat::Compact => match self.device {
                Device::CPU => "-int8",
                Device::CUDA => "-float16",
            },
            ModelFormat::Normal => "",
        };
        match self.model_type {
            JParaCrawlModelType::Small => format!("jparacrawl-small-ct2{}", ending),
            JParaCrawlModelType::Base => format!("jparacrawl-base-ct2{}", ending),
            JParaCrawlModelType::Big => format!("jparacrawl-big-ct2{}", ending),
        }
    }
}

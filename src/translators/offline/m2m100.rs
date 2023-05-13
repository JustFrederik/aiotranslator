use crate::error::Error;
use crate::languages::Language;
use crate::translators::offline::ctranslate2::model_management::{
    CTranslateModels, TokenizerModels,
};
use crate::translators::offline::ctranslate2::Device;
use crate::translators::offline::ModelFormat;
use crate::translators::translator_structure::{TranslationVecOutput, TranslatorCTranslate};
use model_manager::model_manager::ModelManager;
use rustyctranslate2::BatchType;
use std::path::PathBuf;

pub enum M2M100ModelType {
    Small418m,
    Big12b,
}

pub struct M2M100Translator {
    device: Device,
    base_path: PathBuf,
    _model_type: M2M100ModelType,
    model_format: ModelFormat,
}

impl TranslatorCTranslate for M2M100Translator {
    fn translate_vec(
        &self,
        translator_models: &mut CTranslateModels,
        tokenizer_model: &mut TokenizerModels,
        query: &[String],
        _from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let model_path = self.base_path.join("spm.128k.model");
        let tokenizer = tokenizer_model.get_tokenizer("m2m100", model_path)?;
        let tokens = tokenizer.tokenize(query)?;
        let lang_str = to.to_m2m100_str()?;
        let target = Self::generate_target_prefix(&lang_str, query.len());
        let translator = translator_models.get_translator(
            "m2m100",
            self.base_path.clone(),
            &self.device,
            self.model_format.is_compressed(),
        )?;
        let translated = translator
            .translate_batch_target(tokens, None, BatchType::Example, target)
            .map_err(Error::new_option)?;
        let sentences = tokenizer
            .detokenize(translated)?
            .iter()
            .map(|x| x[lang_str.len() + 5..].to_string())
            .collect::<Vec<_>>();
        tokenizer_model.cleanup();
        translator_models.cleanup();
        Ok(TranslationVecOutput {
            text: sentences,
            lang: Language::Unknown,
        })
    }
}

impl M2M100Translator {
    pub async fn new(
        device: Device,
        model_manager: &ModelManager,
        model_format: ModelFormat,
        model_type: M2M100ModelType,
    ) -> Result<Self, Error> {
        let ident = Self::get_model_name(&device, &model_format, &model_type);
        let model = model_manager
            .get_model_async(&ident)
            .await
            .map_err(|_| Error::new_option("couldnt get model".to_string()))?;
        Ok(Self {
            base_path: model.0.join(&model.1.directory),
            device,
            _model_type: model_type,
            model_format: ModelFormat::Normal,
        })
    }

    fn get_model_name(
        device: &Device,
        model_format: &ModelFormat,
        model_type: &M2M100ModelType,
    ) -> String {
        let extra = match model_format {
            ModelFormat::Compact => match device {
                Device::CPU => "_int8",
                Device::CUDA => "_float16"
            }
            ModelFormat::Normal => ""
        };
        format!(
            "m2m_100_{}_ct2{}",
            match model_type {
                M2M100ModelType::Small418m => "418m",
                M2M100ModelType::Big12b => "1.2b",
            }, extra
        )
    }

    fn generate_target_prefix(lang: &str, ammount: usize) -> Vec<String> {
        let mut target_prefix = Vec::new();
        for _ in 0..ammount {
            target_prefix.push(format!("__{}__", lang));
        }
        target_prefix
    }
}

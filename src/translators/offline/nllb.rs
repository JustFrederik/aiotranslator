use crate::error::Error;
use crate::languages::Language;
use crate::translators::offline::ctranslate2::model_management::{
    CTranslateModels, TokenizerModels,
};
use crate::translators::offline::ctranslate2::tokenizer::Tokenizer;
use crate::translators::offline::ctranslate2::Device;
use crate::translators::offline::ModelFormat;
use crate::translators::translator_structure::{TranslationVecOutput, TranslatorCTranslate};
use model_manager::model_manager::ModelManager;
use rustyctranslate2::BatchType;
use std::path::PathBuf;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum NllbModelType {
    #[default]
    DistilledBig1_3B,
    DistilledSmall600M,
    Big3_3B,
    Small1_3B,
}

pub struct NllbTranslator {
    device: Device,
    base_path: PathBuf,
    ident: String,
    model_format: ModelFormat,
}

impl TranslatorCTranslate for NllbTranslator {
    fn translate_vec(
        &self,
        translator_models: &mut CTranslateModels,
        tokenizer_models: &mut TokenizerModels,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let model_path = self.base_path.join("sentencepiece.bpe.model");
        let tokenizer = tokenizer_models.get_tokenizer("m2m100", model_path)?;
        let tokens = Self::tokenize(from, query, tokenizer)?;
        let lang_str = to.to_nllb_str()?;
        let target = Self::generate_target_prefix(&lang_str, query.len());
        let translator = translator_models.get_translator(
            &self.ident,
            self.base_path.clone(),
            &self.device,
            self.model_format.is_compressed(),
        )?;
        let translated = translator
            .translate_batch_target(tokens, None, BatchType::Example, target)
            .map_err(Error::new_option)?;
        let to = to.to_nllb_str()?;
        let sentences = tokenizer
            .detokenize(translated)?
            .iter()
            .map(|x| x[to.len() + 1..].to_string())
            .collect::<Vec<_>>();
        tokenizer_models.cleanup();
        translator_models.cleanup();
        Ok(TranslationVecOutput {
            text: sentences,
            lang: Language::Unknown,
        })
    }
}

impl NllbTranslator {
    pub async fn new(
        device: &Device,
        model_format: &ModelFormat,
        model_type: &NllbModelType,
        model_manager: &ModelManager,
    ) -> Result<Self, Error> {
        let ident = Self::get_ident(device, model_format, model_type);
        let model = model_manager
            .get_model_async(&ident)
            .await
            .map_err(|_| Error::new_option("couldnt get model".to_string()))?;
        let device = *device;
        Ok(Self {
            device,
            base_path: model.0.join(&model.1.directory),
            ident,
            model_format: *model_format,
        })
    }

    pub fn tokenize(
        from: Option<Language>,
        query: &[String],
        tokenizer: &Tokenizer,
    ) -> Result<Vec<Vec<String>>, Error> {
        let source_sentences: Vec<String> = query.iter().map(|s| s.trim().to_string()).collect();
        let tokenized = tokenizer.tokenize(&source_sentences)?;
        let from = from.map(|s| s.to_nllb_str()).unwrap_or(Ok(String::new()))?;
        Ok(tokenized
            .into_iter()
            .map(|s| {
                let mut s = s;
                s.insert(0, from.clone());
                s.push("</s>".to_string());
                s
            })
            .collect())
    }

    fn get_ident(
        device: &Device,
        model_format: &ModelFormat,
        model_type: &NllbModelType,
    ) -> String {
        let extra = match model_format {
            ModelFormat::Compact => match device {
                Device::CPU => "-int8",
                Device::CUDA => "-float16",
            },
            ModelFormat::Normal => "",
        };
        format!(
            "nllb-{}-ct2{}",
            match model_type {
                NllbModelType::DistilledBig1_3B => "200-distilled-1.3B",
                NllbModelType::DistilledSmall600M => "200-distilled-600M",
                NllbModelType::Big3_3B => "200-3.3B",
                NllbModelType::Small1_3B => "200-1.3B",
            },
            extra
        )
    }

    fn generate_target_prefix(lang: &str, ammount: usize) -> Vec<String> {
        let mut target_prefix = Vec::new();
        for _ in 0..ammount {
            target_prefix.push(lang.to_string());
        }
        target_prefix
    }
}

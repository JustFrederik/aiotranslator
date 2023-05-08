use crate::error::Error;
use crate::languages::Language;
use crate::translators::offline::ctranslate2::model_management::Models;
use crate::translators::offline::ctranslate2::py::transalte_with_py;
use crate::translators::offline::ctranslate2::Device;
use crate::translators::translator_structrue::{TranslationVecOutput, TranslatorCTranslate};
use std::path::PathBuf;

pub struct M2M100_12BTranslator {
    device: Device,
    base_path: PathBuf,
}

impl TranslatorCTranslate for M2M100_12BTranslator {
    fn translate_vec(
        &self,
        models: &mut Models,
        query: &[String],
        _from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let model_path = self.base_path.join("sentencepiece.model");
        let tokenizer = models.get_tokenizer("m2m100_1.2b", model_path)?;
        let tokens = tokenizer.tokenize(query)?;
        let lang_str = to.to_m2m100_str()?;
        let prefix = Self::generate_target_prefix(&lang_str, query.len());
        //TODO: replace
        let translated = transalte_with_py(
            self.base_path.clone(),
            tokens,
            Some(prefix),
            &self.device.to_string(),
        )
        .unwrap();
        let sentences = tokenizer
            .detokenize(translated)?
            .iter()
            .map(|x| x[lang_str.len() + 5..].to_string())
            .collect::<Vec<_>>();
        models.cleanup();
        Ok(TranslationVecOutput {
            text: sentences,
            lang: Language::Unknown,
        })
    }
}

impl M2M100_12BTranslator {
    pub fn new(device: Device, base_path: PathBuf) -> Self {
        Self { device, base_path }
    }
    pub(crate) fn generate_target_prefix(lang: &str, ammount: usize) -> Vec<Vec<String>> {
        let mut target_prefix = Vec::new();
        for _ in 0..ammount {
            target_prefix.push(vec![format!("__{}__", lang)]);
        }
        target_prefix
    }
}

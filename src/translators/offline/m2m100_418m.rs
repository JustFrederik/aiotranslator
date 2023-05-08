use crate::error::Error;
use crate::languages::Language;
use crate::translators::offline::ctranslate2::model_management::Models;
use crate::translators::offline::ctranslate2::py::transalte_with_py;
use crate::translators::offline::ctranslate2::Device;
use crate::translators::offline::m2m100_12b::M2M100_12BTranslator;
use crate::translators::translator_structrue::{TranslationVecOutput, TranslatorCTranslate};
use std::path::PathBuf;

pub struct M2M100_418MTranslator {
    device: Device,
    base_path: PathBuf,
}

impl TranslatorCTranslate for M2M100_418MTranslator {
    fn translate_vec(
        &self,
        models: &mut Models,
        query: &[String],
        _from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let model_path = self.base_path.join("sentencepiece.model");
        let tokenizer = models.get_tokenizer("m2m100_418m", model_path)?;
        let tokens = tokenizer.tokenize(query)?;
        let lang_str = to.to_m2m100_str()?;
        let prefix = M2M100_12BTranslator::generate_target_prefix(&lang_str, query.len());
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

impl M2M100_418MTranslator {
    pub fn new(base_path: PathBuf, device: Device) -> Self {
        Self { device, base_path }
    }
}

use crate::error::Error;
use crate::languages::Language;
use crate::translators::offline::ctranslate2::model_management::Models;
use crate::translators::offline::ctranslate2::py::transalte_with_py;
use crate::translators::offline::ctranslate2::Device;
use crate::translators::offline::jparacrawl::JParaCrawlTranslator;
use crate::translators::translator_structrue::{TranslationVecOutput, TranslatorCTranslate};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct JParaCrawlBigTranslator {
    device: Device,
    base_path: PathBuf,
    tokenizer_filenames: HashMap<Language, String>,
}

impl TranslatorCTranslate for JParaCrawlBigTranslator {
    fn translate_vec(
        &self,
        models: &mut Models,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let from = JParaCrawlTranslator::get_from(from, to)?;
        let model_path = self.base_path.join(
            self.tokenizer_filenames
                .get(&from)
                .ok_or_else(|| Error::new_option("Tokenizer not found"))?,
        );
        let tokenizer = models.get_tokenizer(
            &format!("jparacrawl-big-{}", from.to_jparacrawl_str()?),
            model_path,
        )?;
        let tokens = tokenizer.tokenize(query)?;
        //TODO: replace
        let translated = transalte_with_py(
            Self::get_translator_model_path(&self.base_path, from, to)?,
            tokens,
            None,
            &self.device.to_string(),
        )
        .unwrap();
        let sentences = tokenizer.detokenize(translated)?;
        models.cleanup();
        Ok(TranslationVecOutput {
            text: sentences,
            lang: from,
        })
    }
}

impl JParaCrawlBigTranslator {
    pub fn new(base_path: PathBuf, device: Device) -> Self {
        Self {
            device,
            base_path,
            tokenizer_filenames: JParaCrawlTranslator::get_tokenizer_filenames(),
        }
    }

    pub fn get_translator_model_path(
        path: &Path,
        from: Language,
        to: &Language,
    ) -> Result<PathBuf, Error> {
        Ok(path.join(format!(
            "big-{}-{}",
            from.to_jparacrawl_str()?,
            to.to_jparacrawl_str()?
        )))
    }
}

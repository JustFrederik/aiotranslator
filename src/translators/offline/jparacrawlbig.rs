use crate::error::Error;
use crate::languages::Language;
use crate::translators::offline::ctranslate2::model_management::Models;
use crate::translators::offline::ctranslate2::py::transalte_with_py;
use crate::translators::offline::jparacrawl::JParaCrawlTranslator;
use crate::translators::translator_structrue::TranslationVecOutput;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct JParaCrawlBigTranslator {
    base_path: PathBuf,
    tokenizer_filenames: HashMap<Language, String>,
}

impl JParaCrawlBigTranslator {
    pub fn new(base_path: PathBuf) -> Self {
        let mut tokenizer_filenames = HashMap::new();
        tokenizer_filenames.insert(Language::English, "spm.en.nopretok.model".to_string());
        tokenizer_filenames.insert(Language::Japanese, "spm.ja.nopretok.model".to_string());
        Self {
            base_path,
            tokenizer_filenames,
        }
    }

    pub fn translate_vec(
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
            &format!("jparacrawl-big-{}", from.to_6393_str().unwrap()),
            model_path,
        )?;
        let tokens = tokenizer.tokenize(query)?;
        let translated = transalte_with_py(
            Self::get_translator_model_path(&self.base_path, from, to),
            tokens,
            "cpu".to_string(),
        )
        .unwrap();
        let sentences = tokenizer.detokenize(translated)?;
        models.cleanup();
        Ok(TranslationVecOutput {
            text: sentences,
            lang: from,
        })
    }
    pub(crate) fn get_translator_model_path(path: &Path, from: Language, to: &Language) -> PathBuf {
        path.join(format!(
            "big-{}-{}",
            from.to_6391_str().unwrap(),
            to.to_6391_str().unwrap()
        ))
    }
}

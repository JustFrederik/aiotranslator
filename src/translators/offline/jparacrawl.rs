use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::languages::Language;
use crate::translators::offline::ctranslate2::model_management::Models;
use crate::translators::offline::ctranslate2::py::transalte_with_py;
use crate::translators::translator_structrue::TranslationVecOutput;

pub struct JParaCrawlTranslator {
    base_path: PathBuf,
    tokenizer_filenames: HashMap<Language, String>,
}

impl JParaCrawlTranslator {
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
        let from = Self::get_from(from, to)?;
        let model_path = self.base_path.join(
            self.tokenizer_filenames
                .get(&from)
                .ok_or_else(|| Error::new_option("Tokenizer not found"))?,
        );
        let tokenizer = models.get_tokenizer(
            &format!("jparacrawl-{}", from.to_6393_str().unwrap()),
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

    pub fn get_translator_model_path(path: &Path, from: Language, to: &Language) -> PathBuf {
        path.join(format!(
            "base-{}-{}",
            from.to_6391_str().unwrap(),
            to.to_6391_str().unwrap()
        ))
    }

    pub(crate) fn get_from(from: Option<Language>, to: &Language) -> Result<Language, Error> {
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
}

use crate::error::Error;
use crate::languages::Language;
use crate::translators::offline::ctranslate2::model_management::Models;
use crate::translators::offline::ctranslate2::py::transalte_with_py;
use crate::translators::offline::ctranslate2::Device;
use crate::translators::offline::jparacrawlbig::JParaCrawlBigTranslator;
use crate::translators::translator_structrue::{TranslationVecOutput, TranslatorCTranslate};
use std::path::PathBuf;

pub struct SugoiTranslator {
    device: Device,
    base_path: PathBuf,
}

impl TranslatorCTranslate for SugoiTranslator {
    fn translate_vec(
        &self,
        models: &mut Models,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let from = Self::get_from(from, to)?;
        let model_path = self.base_path.join("spm.ja.nopretok.model");
        let tokenizer =
            models.get_tokenizer(&format!("sugoi-{}", from.to_jparacrawl_str()?), model_path)?;
        let (query, query_split_sizes) = Self::pre_tokenize(query);
        let tokens = tokenizer.tokenize(&query)?;
        //TODO: replace
        let translated = transalte_with_py(
            JParaCrawlBigTranslator::get_translator_model_path(&self.base_path, from, to)?,
            tokens,
            None,
            &self.device.to_string(),
        )
        .unwrap();
        let sentences = tokenizer.detokenize(translated)?;
        let sentences = Self::post_detokenize(sentences, query_split_sizes);
        models.cleanup();
        Ok(TranslationVecOutput {
            text: sentences,
            lang: from,
        })
    }
}

impl SugoiTranslator {
    pub fn new(base_path: PathBuf, device: Device) -> Self {
        Self { device, base_path }
    }

    fn get_from(from: Option<Language>, to: &Language) -> Result<Language, Error> {
        if let Some(f) = from {
            if f == Language::Japanese && to == &Language::English {
                return Ok(f);
            }
            Err(Error::new_option("Language not supported"))?
        } else {
            Ok(match to {
                Language::English => Language::Japanese,
                _ => Err(Error::new_option("Language not supported"))?,
            })
        }
    }

    fn pre_tokenize(queries: &[String]) -> (Vec<String>, Vec<usize>) {
        let mut new_queries: Vec<String> = Vec::new();
        let mut query_split_sizes: Vec<usize> = Vec::new();
        for q in queries {
            //Regex cant fail
            let re = regex::Regex::new(r"(\w[.‥…!?。・]+)").unwrap();
            let sentences: Vec<&str> = re.split(q).collect();
            let mut chunk_queries: Vec<String> = Vec::new();
            for chunk in sentences.chunks(4) {
                let s = chunk.join("");
                chunk_queries.push(s.replace(|c: char| c == '.' || c == '。', "@"));
            }
            query_split_sizes.push(chunk_queries.len());
            new_queries.extend(chunk_queries);
        }
        (new_queries, query_split_sizes)
    }

    fn post_detokenize(translations: Vec<String>, query_split_sizes: Vec<usize>) -> Vec<String> {
        let mut new_translations: Vec<String> = Vec::new();
        let mut i = 0;
        for query_count in query_split_sizes {
            let sentences = translations[i..i + query_count].join(" ");
            i += query_count;
            let sentences = sentences
                .replace('@', ".")
                .replace('▁', " ")
                .replace("<unk>", "");
            new_translations.push(sentences);
        }
        new_translations
    }
}

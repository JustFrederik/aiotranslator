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

pub struct SugoiTranslator {
    device: Device,
    base_path: PathBuf,
    model_format: ModelFormat,
    ident: String,
}

impl TranslatorCTranslate for SugoiTranslator {
    fn translate_vec(
        &self,
        translator_models: &mut CTranslateModels,
        tokenizer_model: &mut TokenizerModels,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let from = Self::get_from(from, to)?;
        let model_path = self.base_path.join("spm.ja.nopretok.model");
        let tokenizer = tokenizer_model.get_tokenizer("sugoi", model_path)?;
        let (query, query_split_sizes) = Self::pre_tokenize(query);
        let tokens = tokenizer.tokenize(&query)?;
        let translator = translator_models.get_translator(
            &self.ident,
            self.base_path.clone(),
            &self.device,
            self.model_format.is_compressed(),
        )?;
        let translated = translator
            .translate_batch(tokens, None, BatchType::Example)
            .map_err(Error::new_option)?;
        let sentences = tokenizer.detokenize(translated)?;
        let sentences = Self::post_detokenize(sentences, query_split_sizes);
        tokenizer_model.cleanup();
        Ok(TranslationVecOutput {
            text: sentences,
            lang: from,
        })
    }
}

impl SugoiTranslator {
    pub async fn new(
        device: &Device,
        model_format: &ModelFormat,
        mm: &ModelManager,
    ) -> Result<Self, Error> {
        let ident = Self::get_model_name(device, model_format);
        let model = mm
            .get_model_async(&ident)
            .await
            .map_err(|_| Error::new_option("couldnt get model".to_string()))?;
        Ok(Self {
            ident,
            device: *device,
            base_path: model.0.join(&model.1.directory),
            model_format: *model_format,
        })
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

    fn get_model_name(device: &Device, model_format: &ModelFormat) -> String {
        format!(
            "sugoi-ja-en-ct2{}",
            match model_format {
                ModelFormat::Compact => match device {
                    Device::CPU => "-int8",
                    Device::CUDA => "-float16",
                },
                ModelFormat::Normal => "",
            }
        )
    }
}

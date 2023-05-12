use model_manager::model_manager::{HuggingfaceModel, Model, ModelManager, ModelSource};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

#[allow(dead_code)]
pub fn register(mm: &mut ModelManager) {
    let mut models = HashMap::new();
    let jpara = [
        "spm.ja.nopretok.vocab",
        "spm.ja.nopretok.model",
        "spm.en.nopretok.vocab",
        "spm.en.nopretok.model",
        "ja-en/config.json",
        "ja-en/model.bin",
        "ja-en/source_vocabulary.txt",
        "ja-en/target_vocabulary.txt",
        "en-ja/config.json",
        "en-ja/model.bin",
        "en-ja/source_vocabulary.txt",
        "en-ja/target_vocabulary.txt",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect::<Vec<_>>();
    let sugoi = [
        "source_vocabulary.txt",
        "target_vocabulary.txt",
        "spm.ja.nopretok.vocab",
        "spm.ja.nopretok.model",
        "model.bin",
        "config.json",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect::<Vec<_>>();
    models.insert(
        "jparacrawl-small-ct2".to_string(),
        Model {
            directory: PathBuf::from_str("translators/jparacrawl-small-ct2").unwrap(),
            version: 3.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/jparacrawl-v3-small-ct2".to_string(),
                files: jpara.clone(),
                commit: None,
            }),
        },
    );
    models.insert(
        "jparacrawl-small-ct2-int8".to_string(),
        Model {
            directory: PathBuf::from_str("translators/jparacrawl-small-ct2-int8").unwrap(),
            version: 3.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/jparacrawl-v3-small-ct2-int8".to_string(),
                files: jpara.clone(),
                commit: None,
            }),
        },
    );
    models.insert(
        "jparacrawl-small-ct2-float16".to_string(),
        Model {
            directory: PathBuf::from_str("translators/jparacrawl-small-ct2-float16").unwrap(),
            version: 3.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/jparacrawl-v3-small-ct2-float16".to_string(),
                files: jpara.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "jparacrawl")]
    models.insert(
        "jparacrawl-base-ct2".to_string(),
        Model {
            directory: PathBuf::from_str("translators/jparacrawl-base-ct2").unwrap(),
            version: 3.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/jparacrawl-v3-base-ct2".to_string(),
                files: jpara.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "jparacrawl")]
    models.insert(
        "jparacrawl-base-ct2-int8".to_string(),
        Model {
            directory: PathBuf::from_str("translators/jparacrawl-base-ct2-int8").unwrap(),
            version: 3.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/jparacrawl-v3-base-ct2-int8".to_string(),
                files: jpara.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "jparacrawl")]
    models.insert(
        "jparacrawl-base-ct2-float16".to_string(),
        Model {
            directory: PathBuf::from_str("translators/jparacrawl-base-ct2-float16").unwrap(),
            version: 3.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/jparacrawl-v3-base-ct2-float16".to_string(),
                files: jpara.clone(),
                commit: None,
            }),
        },
    );
    models.insert(
        "jparacrawl-big-ct2".to_string(),
        Model {
            directory: PathBuf::from_str("translators/jparacrawl-big-ct2").unwrap(),
            version: 3.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/jparacrawl-v3-big-ct2".to_string(),
                files: jpara.clone(),
                commit: None,
            }),
        },
    );
    models.insert(
        "jparacrawl-big-ct2-int8".to_string(),
        Model {
            directory: PathBuf::from_str("translators/jparacrawl-big-ct2-int8").unwrap(),
            version: 3.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/jparacrawl-v3-big-ct2-int8".to_string(),
                files: jpara.clone(),
                commit: None,
            }),
        },
    );
    models.insert(
        "jparacrawl-big-ct2-float16".to_string(),
        Model {
            directory: PathBuf::from_str("translators/jparacrawl-big-ct2-float16").unwrap(),
            version: 3.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/jparacrawl-v3-big-ct2-float16".to_string(),
                files: jpara,
                commit: None,
            }),
        },
    );
    #[cfg(feature = "sugoi")]
    models.insert(
        "sugoi-ja-en-ct2".to_string(),
        Model {
            directory: PathBuf::from_str("translators/sugoi-v4-ja-en-ct2").unwrap(),
            version: 4.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/sugoi-v4-ja-en-ct2".to_string(),
                files: sugoi.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "sugoi")]
    models.insert(
        "sugoi-ja-en-ct2-int8".to_string(),
        Model {
            directory: PathBuf::from_str("translators/sugoi-v4-ja-en-ct2-int8").unwrap(),
            version: 4.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/sugoi-v4-ja-en-ct2-int8".to_string(),
                files: sugoi.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "sugoi")]
    models.insert(
        "sugoi-ja-en-ct2-float16".to_string(),
        Model {
            directory: PathBuf::from_str("translators/sugoi-v4-ja-en-ct2-float16").unwrap(),
            version: 4.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "o0Frederik0o/sugoi-v4-ja-en-ct2-float16".to_string(),
                files: sugoi,
                commit: None,
            }),
        },
    );
    models.insert("m2m100-418m-ct2".to_string(), Model{
        directory: PathBuf::from_str("translators/m2m100-418m-ct2").unwrap(),
        version: "14/03/2023".to_string(),
        source: ModelSource::Zip("https://github.com/zyddnys/manga-image-translator/releases/download/beta-0.3/m2m100_418m_ct2.zip".to_string())
    });
    models.insert("m2m100-1.2b-ct2".to_string(), Model{
        directory: PathBuf::from_str("translators/m2m100-1.2b").unwrap(),
        version: "14/03/2023".to_string(),
        source: ModelSource::Zip("https://github.com/zyddnys/manga-image-translator/releases/download/beta-0.3/m2m100_12b_ct2.zip".to_string()),
    });
    mm.register_models(models);
}

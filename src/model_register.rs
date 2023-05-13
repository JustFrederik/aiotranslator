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
    let m2m100 = [
        "spm.128k.model",
        "shared_vocabulary.txt",
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
                repo: "JustFrederik/jparacrawl-v3-small-ct2".to_string(),
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
                repo: "JustFrederik/jparacrawl-v3-small-ct2-int8".to_string(),
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
                repo: "JustFrederik/jparacrawl-v3-small-ct2-float16".to_string(),
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
                repo: "JustFrederik/jparacrawl-v3-base-ct2".to_string(),
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
                repo: "JustFrederik/jparacrawl-v3-base-ct2-int8".to_string(),
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
                repo: "JustFrederik/jparacrawl-v3-base-ct2-float16".to_string(),
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
                repo: "JustFrederik/jparacrawl-v3-big-ct2".to_string(),
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
                repo: "JustFrederik/jparacrawl-v3-big-ct2-int8".to_string(),
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
                repo: "JustFrederik/jparacrawl-v3-big-ct2-float16".to_string(),
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
                repo: "JustFrederik/sugoi-v4-ja-en-ct2".to_string(),
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
                repo: "JustFrederik/sugoi-v4-ja-en-ct2-int8".to_string(),
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
                repo: "JustFrederik/sugoi-v4-ja-en-ct2-float16".to_string(),
                files: sugoi,
                commit: None,
            }),
        },
    );
    #[cfg(feature = "m2m100")]
    models.insert(
        "m2m_100_418m_ct2".to_string(),
        Model {
            directory: PathBuf::from_str("translators/m2m_100_418m_ct2").unwrap(),
            version: 4.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "JustFrederik/m2m_100_418m_ct2".to_string(),
                files: m2m100.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "m2m100")]
    models.insert(
        "m2m_100_418m_ct2_float16".to_string(),
        Model {
            directory: PathBuf::from_str("translators/m2m_100_418m_ct2_float16").unwrap(),
            version: 4.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "JustFrederik/m2m_100_418m_ct2_float16".to_string(),
                files: m2m100.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "m2m100")]
    models.insert(
        "m2m_100_418m_ct2_int8".to_string(),
        Model {
            directory: PathBuf::from_str("translators/m2m_100_418m_ct2_int8").unwrap(),
            version: 4.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "JustFrederik/m2m_100_418m_ct2_int8".to_string(),
                files: m2m100.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "m2m100")]
    models.insert(
        "m2m_100_1.2b_ct2".to_string(),
        Model {
            directory: PathBuf::from_str("translators/m2m_100_1.2b_ct2").unwrap(),
            version: 4.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "JustFrederik/m2m_100_1.2b_ct2".to_string(),
                files: m2m100.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "m2m100")]
    models.insert(
        "m2m_100_1.2b_ct2_int8".to_string(),
        Model {
            directory: PathBuf::from_str("translators/m2m_100_1.2b_ct2_int8").unwrap(),
            version: 4.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "JustFrederik/m2m_100_1.2b_ct2_int8".to_string(),
                files: m2m100.clone(),
                commit: None,
            }),
        },
    );
    #[cfg(feature = "m2m100")]
    models.insert(
        "m2m_100_1.2b_ct2_float16".to_string(),
        Model {
            directory: PathBuf::from_str("translators/m2m_100_1.2b_ct2_float16").unwrap(),
            version: 4.0.to_string(),
            source: ModelSource::Huggingface(HuggingfaceModel {
                repo: "JustFrederik/m2m_100_1.2b_ct2_float16".to_string(),
                files: m2m100,
                commit: None,
            }),
        },
    );
    mm.register_models(models);
}

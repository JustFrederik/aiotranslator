#[cfg(feature = "offline_req")]
use std::collections::HashMap;
#[cfg(feature = "offline_req")]
use std::path::PathBuf;
#[cfg(feature = "offline_req")]
use std::str::FromStr;

#[cfg(feature = "offline_req")]
use model_manager::model_manager::{Model, ModelManager};

pub mod detector;
pub mod error;
#[cfg(feature = "generate")]
pub mod generator;
mod languages;
pub mod translators;

#[allow(dead_code)]
#[cfg(feature = "offline_req")]
fn register(mm: &mut ModelManager) {
    let mut models = HashMap::new();
    #[cfg(feature = "jparacrawl")]
    models.insert("JParaCrawl".to_string(), Model{
        url: "https://github.com/zyddnys/manga-image-translator/releases/download/beta-0.3/jparacrawl-base-models.zip".to_string(),
        directory: PathBuf::from_str("translators/jparacrawl/base").unwrap(),
        version: 3.0,
    });
    #[cfg(feature = "jparacrawlbig")]
    models.insert("JParaCrawlBig".to_string(), Model{
        url: "https://github.com/zyddnys/manga-image-translator/releases/download/beta-0.3/jparacrawl-big-models.zip".to_string(),
        directory: PathBuf::from_str("translators/jparacrawl/big").unwrap(),
        version: 3.0,
    });
    #[cfg(feature = "sugoi")]
    models.insert("Sugoi".to_string(), Model{
        url: "https://github.com/zyddnys/manga-image-translator/releases/download/beta-0.3/sugoi-models.zip".to_string(),
        directory: PathBuf::from_str("translators/sugoi").unwrap(),
        version: 4.0,
    });
    models.insert("M2M100 418m".to_string(), Model{
        url: "https://github.com/zyddnys/manga-image-translator/releases/download/beta-0.3/m2m100_418m_ct2.zip".to_string(),
        directory: PathBuf::from_str("translators/m2m100/418m").unwrap(),
        version: 0.01,
    });
    models.insert("M2M100 1.2b".to_string(), Model{
        url: "https://github.com/zyddnys/manga-image-translator/releases/download/beta-0.3/m2m100_12b_ct2.zip".to_string(),
        directory: PathBuf::from_str("translators/m2m100/1.2b").unwrap(),
        version: 0.01,
    });
    mm.register_models(models);
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dotenv::dotenv;
    #[cfg(feature = "offline_req")]
    use model_manager::model_manager::ModelManager;
    use reqwest::Client;

    use crate::detector;
    use crate::detector::Detectors;
    use crate::generator::Records;
    use crate::languages::Language;
    #[cfg(feature = "offline_req")]
    use crate::register;
    use crate::translators::chainer::{TranslatorInfo, TranslatorSelectorInfo};
    use crate::translators::context::Context;
    use crate::translators::dev::{get_csv_errors, get_languages};
    use crate::translators::scrape::papago::PapagoTranslator;
    use crate::translators::tokens::Tokens;
    use crate::translators::translator_structrue::TranslatorLanguages;
    use crate::translators::{Translator, Translators};

    #[tokio::test]
    #[cfg(feature = "offline_req")]
    async fn models() {
        let mut mm = ModelManager::new().unwrap();
        register(&mut mm);
        mm.proccess(3).await.unwrap();
        mm.remove_zips().unwrap();
        mm.clean_directory().unwrap();
    }

    #[test]
    fn test_detector() {
        let text = "Hallo Welt";
        let lang = detector::detect_language(text, &Detectors::Whatlang).unwrap();
        println!("{:?}", lang);
    }

    #[tokio::test]
    async fn test_get_languages() {
        dotenv().ok();
        let res = get_languages(&Translator::Bing, &Tokens::get_env().unwrap())
            .await
            .unwrap();
        print!("{:?}", res);
    }

    #[tokio::test]
    async fn translate() {
        dotenv().ok();
        let mut hashmap = HashMap::new();
        hashmap.insert(Language::Chinese, Translator::Papago);
        let selector = TranslatorSelectorInfo::Selective(
            hashmap,
            TranslatorInfo {
                translator: Translator::Google,
                to: Language::English,
            },
        );
        let v = Translators::new(
            Some(Tokens::get_env().unwrap()),
            selector,
            None,
            Some(3),
            Detectors::Whatlang,
        )
        .await
        .unwrap();
        let chatgpt_context = Context::ChatGPT("This is a text about ...".to_string());
        let translation = v
            .translate("Hello world".to_string(), None, &[chatgpt_context])
            .await
            .unwrap();
        let translations = v
            .translate_vec(
                vec!["Hello world".to_string(), "This is a test".to_string()],
                None,
                &[],
            )
            .await
            .unwrap();
        println!("{:?}, {:?}", translation, translations);
    }

    #[tokio::test]
    async fn generate_file() {
        let v = Records::new().unwrap();
        v.generate_file().unwrap();
    }

    #[tokio::test]
    async fn add_line() {
        dotenv().ok();
        let vv = PapagoTranslator::get_languages(&Client::new(), &Tokens::get_env().unwrap())
            .await
            .unwrap();
        //println!("{:?}", vv);
        let mut v = Records::new().unwrap();
        // v.add_line("nllb", &vv);
        v.write_file("test.csv").expect("TODO: panic message");
    }

    #[tokio::test]
    async fn get_csv_error() {
        dotenv().ok();
        let v = get_csv_errors(Translator::Papago, &Tokens::get_env().unwrap())
            .await
            .unwrap();
        println!("{:#?}", v);
    }
}

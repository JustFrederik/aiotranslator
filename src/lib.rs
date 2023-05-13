pub mod detector;
pub mod error;
#[cfg(feature = "generate")]
pub mod generator;
mod languages;
#[cfg(feature = "offline_req")]
mod model_register;
pub mod translators;

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
    use crate::model_register::register;
    use crate::translators::chainer::{TranslatorInfo, TranslatorSelectorInfo};
    use crate::translators::context::Context;
    use crate::translators::dev::{get_csv_errors, get_languages};
    use crate::translators::scrape::papago::PapagoTranslator;
    use crate::translators::tokens::Tokens;
    use crate::translators::translator_structure::TranslatorLanguages;
    use crate::translators::{Translator, Translators};

    #[tokio::test]
    #[cfg(feature = "offline_req")]
    async fn models() {
        let mut mm = ModelManager::new().unwrap();
        register(&mut mm);
        mm.download_all(3).await.unwrap();
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
        v.add_line("nllb", &vv);
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

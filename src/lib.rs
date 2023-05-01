pub mod detector;
pub mod error;
#[cfg(feature = "generate")]
pub mod generator;
mod languages;
pub mod translators;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use dotenv::dotenv;
    use edge_gpt::ConversationStyle;
    use reqwest::Client;

    use crate::detector;
    use crate::detector::Detectors;
    use crate::generator::Records;
    use crate::languages::Language;
    use crate::translators::chainer::{TranslatorInfo, TranslatorSelectorInfo};
    use crate::translators::dev::{get_csv_errors, get_languages};
    use crate::translators::scrape::papago::PapagoTranslator;
    use crate::translators::tokens::Tokens;
    use crate::translators::translator_structrue::{TranslatorContext, TranslatorLanguages};
    use crate::translators::{Translator, Translators};

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
        let selector = TranslatorSelectorInfo::Selective(hashmap,  TranslatorInfo {
            translator: Translator::Google,
            to: Language::English,
        });
        let v = Translators::new(
            Some(Tokens::get_env().unwrap()),
            selector,
            None,
            Some(3),
            Detectors::Whatlang,
        )
        .await
        .unwrap();
        println!("{:?}", v.translators);
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
        println!("{:?}", vv);
        let mut v = Records::new().unwrap();
        v.add_line("papago", &vv);
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

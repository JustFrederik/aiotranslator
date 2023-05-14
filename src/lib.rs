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
    use crate::translators::dev::{get_csv_errors, get_languages};
    use crate::translators::offline::ctranslate2::model_management::{
        CTranslateModels, ModelLifetime, TokenizerModels,
    };
    use crate::translators::offline::ctranslate2::Device;
    use crate::translators::offline::m2m100::{M2M100ModelType, M2M100Translator};
    use crate::translators::offline::ModelFormat;
    use crate::translators::scrape::papago::PapagoTranslator;
    use crate::translators::tokens::Tokens;
    use crate::translators::translator_structure::{TranslatorCTranslate, TranslatorLanguages};
    use crate::translators::Translator;

    #[tokio::test]
    #[cfg(feature = "offline_req")]
    async fn translate_offline() {
        let time = std::time::Instant::now();
        let mut mm = ModelManager::new().unwrap();
        register(&mut mm);
        let v = M2M100Translator::new(
            &Device::CPU,
            &ModelFormat::Normal,
            &M2M100ModelType::Small418m,
            &mm,
        )
        .await
        .unwrap();
        let mut tk = TokenizerModels::new(ModelLifetime::KeepAlive);
        let mut tt = CTranslateModels::new(ModelLifetime::KeepAlive);

        let mess = v
            .translate_vec(
                &mut tt,
                &mut tk,
                &["こんにちは!".to_string()],
                None,
                &Language::English,
            )
            .unwrap();
        println!("{:?}", mess);
        println!("{:?}", time.elapsed());
    }

    #[tokio::test]
    #[cfg(feature = "offline_req")]
    async fn models() {
        //TODO: better downloader https://github.com/mattgathu/duma/tree/master
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
    #[cfg(not(feature = "offline_req"))]
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

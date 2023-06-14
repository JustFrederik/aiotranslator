use std::str::FromStr;

use reqwest::blocking::Client;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::api::deepl::DeeplTranslator;
use crate::translators::api::libretranslate::LibreTranslateTranslator;
use crate::translators::api::mymemory::MyMemoryTranslator;
use crate::translators::scrape::baidu::BaiduTranslator;
use crate::translators::scrape::bing::BingTranslator;
use crate::translators::scrape::google::GoogleTranslator;
use crate::translators::scrape::papago::PapagoTranslator;
use crate::translators::scrape::youdao::YoudaoTranslator;
use crate::translators::tokens::Tokens;
use crate::translators::translator_structure::TranslatorLanguages;
use crate::translators::Translator;

/// Returns the available languages for selected translator.
/// These langauges are fetched from the web.
#[cfg(feature = "fetch_languages")]
pub async fn get_languages(translator: &Translator, tokens: &Tokens) -> Result<Vec<String>, Error> {
    let client = Client::new();
    match translator {
        #[cfg(feature = "chatgpt")]
        Translator::ChatGPT(_, _, _, _, _) => Ok(vec![]),
        #[cfg(feature = "bing-scrape")]
        Translator::Bing => BingTranslator::get_languages(&client, tokens),
        Translator::Google => GoogleTranslator::get_languages(&client, tokens),
        #[cfg(feature = "deepl")]
        Translator::Deepl => DeeplTranslator::get_languages(&client, tokens),
        #[cfg(feature = "mymemory")]
        Translator::MyMemory => MyMemoryTranslator::get_languages(&client, tokens),
        #[cfg(feature = "libre")]
        Translator::LibreTranslate => LibreTranslateTranslator::get_languages(&client, tokens),
        #[cfg(feature = "papago-scrape")]
        Translator::Papago => PapagoTranslator::get_languages(&client, tokens),
        #[cfg(feature = "youdao-scrape")]
        Translator::Youdao(_) => YoudaoTranslator::get_languages(&client, tokens),
        #[cfg(feature = "baidu-scrape")]
        Translator::Baidu(_) => BaiduTranslator::get_languages(&client, tokens),
        Translator::EdgeGPT(_, _) => Ok(vec![]),
        #[cfg(feature = "nllb")]
        Translator::Nllb(_, _, _) => unimplemented!(),
        #[cfg(feature = "m2m100")]
        Translator::M2M100(_, _, _) => unimplemented!(),
        #[cfg(feature = "jparacrawl")]
        Translator::JParaCrawl(_, _, _) => unimplemented!(),
        #[cfg(feature = "sugoi")]
        Translator::Sugoi(_, _) => unimplemented!(),
    }
}

pub async fn get_csv_errors(
    translator: Translator,
    tokens: &Tokens,
) -> Result<(Vec<String>, Vec<String>), Error> {
    let langs = get_languages(&translator, tokens).await?;

    let to_str = |v: &Language| -> Result<String, Error> {
        match &translator {
            Translator::Deepl => v.to_deepl_str(),
            Translator::ChatGPT(_, _, _, _, _) | Translator::EdgeGPT(_, _) => Err(
                Error::new_option("ChatGPT does not support language detection"),
            ),
            Translator::Google => v.to_google_str(),
            Translator::Bing => v.to_bing_str(),
            Translator::LibreTranslate => v.to_libretranslate_str(),
            Translator::MyMemory => v.to_mymemory_str(),
            Translator::Papago => v.to_papago_str(),
            Translator::Youdao(_) => v.to_youdao_str(),
            Translator::Baidu(_) => v.to_baidu_str(),
            #[cfg(feature = "nllb")]
            Translator::Nllb(_, _, _) => v.to_nllb_str(),
            #[cfg(feature = "m2m100")]
            Translator::M2M100(_, _, _) => v.to_m2m100_str(),
            #[cfg(feature = "jparacrawl")]
            Translator::JParaCrawl(_, _, _) => v.to_jparacrawl_str(),
            #[cfg(feature = "sugoi")]
            Translator::Sugoi(_, _) => v.to_sugoi_str(),
        }
    };

    let get_lang = |v: &str| -> Result<(), Error> {
        match &translator {
            Translator::ChatGPT(_, _, _, _, _) => Err(Error::new_option(
                "ChatGPT does not support language detection",
            )),
            _ => match to_str(&Language::from_str(v)?)? == v {
                true => Ok(()),
                false => Err(Error::new_option("Language is not equal")),
            },
        }
    };
    let get_supported = match translator {
        Translator::Deepl => Language::get_supported_deepl(),
        Translator::ChatGPT(_, _, _, _, _) | Translator::EdgeGPT(_, _) => vec![],
        Translator::Google => Language::get_supported_google(),
        Translator::Bing => Language::get_supported_bing(),
        Translator::LibreTranslate => Language::get_supported_libretranslate(),
        Translator::MyMemory => Language::get_supported_mymemory(),
        Translator::Papago => Language::get_supported_papago(),
        Translator::Youdao(_) => Language::get_supported_youdao(),
        Translator::Baidu(_) => Language::get_supported_baidu(),
        #[cfg(feature = "nllb")]
        Translator::Nllb(_, _, _) => Language::get_supported_nllb(),
        #[cfg(feature = "m2m100")]
        Translator::M2M100(_, _, _) => Language::get_supported_m2m100(),
        #[cfg(feature = "jparacrawl")]
        Translator::JParaCrawl(_, _, _) => Language::get_supported_jparacrawl(),
        #[cfg(feature = "sugoi")]
        Translator::Sugoi(_, _) => Language::get_supported_sugoi(),
    };

    let missing = langs
        .iter()
        .map(|v| (v.to_string(), get_lang(v)))
        .filter_map(|v| match v.1 {
            Ok(_) => None,
            Err(_) => Some(v.0),
        })
        .collect::<Vec<_>>();
    let server_removed = get_supported.into_iter().map(|v| to_str(&v));
    let mut s_r = vec![];
    for v in server_removed {
        s_r.push(v?);
    }
    let server_removed = s_r
        .into_iter()
        .filter(|v| !langs.contains(v))
        .collect::<Vec<_>>();
    Ok((missing, server_removed))
}

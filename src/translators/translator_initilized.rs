#[cfg(feature = "offline_req")]
use model_manager::model_manager::ModelManager;
use reqwest::Client;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::api::chatgpt::ChatGPTTranslator;
use crate::translators::api::deepl::DeeplTranslator;
use crate::translators::api::libretranslate::LibreTranslateTranslator;
use crate::translators::api::mymemory::MyMemoryTranslator;
use crate::translators::chainer::TranslatorInfo;
#[cfg(feature = "jparacrawl")]
use crate::translators::offline::jparacrawl::JParaCrawlTranslator;
#[cfg(feature = "m2m100")]
use crate::translators::offline::m2m100::M2M100Translator;
#[cfg(feature = "nllb")]
use crate::translators::offline::nllb::NllbTranslator;
#[cfg(feature = "sugoi")]
use crate::translators::offline::sugoi::SugoiTranslator;
use crate::translators::scrape::baidu::BaiduTranslator;
use crate::translators::scrape::bing::BingTranslator;
use crate::translators::scrape::edgegpt::EdgeGpt;
use crate::translators::scrape::google::GoogleTranslator;
use crate::translators::scrape::papago::PapagoTranslator;
use crate::translators::scrape::youdao::YoudaoTranslator;
use crate::translators::tokens::Tokens;
use crate::translators::translator_structure::TranslatorDyn;
use crate::translators::Translator;

#[derive(Debug)]
pub struct TranslatorInitialized {
    pub data: TranslatorDyn,
    pub translator: Translator,
    pub to: Language,
}

impl TranslatorInitialized {
    pub async fn new(
        info: TranslatorInfo,
        tokens: &Tokens,
        client: &Client,
        #[cfg(feature = "offline_req")] model_manager: &ModelManager,
    ) -> Result<Self, Error> {
        let data: TranslatorDyn = match &info.translator {
            Translator::Deepl => {
                let deepl_token = tokens
                    .deepl_token
                    .as_ref()
                    .ok_or_else(|| Error::new_option("No deepl token"))?;
                TranslatorDyn::NC(Box::new(DeeplTranslator::new(deepl_token)))
            }
            Translator::ChatGPT(model, op, p, temp) => {
                let chat_gpt_token = tokens
                    .gpt_token
                    .as_ref()
                    .ok_or_else(|| Error::new_option("No gpt token"))?;
                TranslatorDyn::WC(Box::new(ChatGPTTranslator::new(
                    model,
                    chat_gpt_token,
                    p,
                    op,
                    *temp,
                )?))
            }
            Translator::Google => TranslatorDyn::NC(Box::new(GoogleTranslator::new())),
            Translator::Bing => TranslatorDyn::NC(Box::new(BingTranslator::new(client).await?)),
            Translator::LibreTranslate => {
                TranslatorDyn::NC(Box::new(LibreTranslateTranslator::new(&tokens.libre_token)))
            }
            Translator::MyMemory => TranslatorDyn::NC(Box::new(MyMemoryTranslator::new())),
            Translator::Papago => TranslatorDyn::NC(Box::new(PapagoTranslator::new())),
            Translator::Youdao => TranslatorDyn::NC(Box::new(YoudaoTranslator::new())),
            Translator::Baidu => TranslatorDyn::NC(Box::new(BaiduTranslator::new())),
            Translator::EdgeGPT(csc, path) => {
                TranslatorDyn::WC(Box::new(EdgeGpt::new(csc, path).await?))
            }
            #[cfg(feature = "nllb")]
            Translator::Nllb(device, model_format, model_type) => TranslatorDyn::Of(Box::new(
                NllbTranslator::new(device, model_format, model_type, model_manager).await?,
            )),
            #[cfg(feature = "m2m100")]
            Translator::M2M100(device, model_format, model_type) => TranslatorDyn::Of(Box::new(
                M2M100Translator::new(device, model_format, model_type, model_manager).await?,
            )),
            #[cfg(feature = "jparacrawl")]
            Translator::JParaCrawl(device, model_format, model_type) => {
                TranslatorDyn::Of(Box::new(
                    JParaCrawlTranslator::new(device, model_format, model_type, model_manager)
                        .await?,
                ))
            }
            #[cfg(feature = "sugoi")]
            Translator::Sugoi(device, model_format) => TranslatorDyn::Of(Box::new(
                SugoiTranslator::new(device, model_format, model_manager).await?,
            )),
        };
        Ok(Self {
            data,
            translator: info.translator,
            to: info.to,
        })
    }
}

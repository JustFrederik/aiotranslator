use crate::error::Error;
use crate::languages::Language;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorNoContext,
};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

struct PapagoApiTranslator {
    client_id: String,
    client_secret: String,
}

impl TranslatorNoContext for PapagoApiTranslator {
    fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> std::result::Result<TranslationOutput, Error> {
        let data = PapagoRequest {
            source: from
                .map(|v| v.to_papago_str())
                .unwrap_or_else(|| Ok("auto".to_string()))?,
            target: to.to_papago_str()?,
            text: query.to_string(),
        };
        let res: PapagoResponse = client
            .post("https://openapi.naver.com/v1/papago/n2mt")
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("X-Naver-Client-Id", &self.client_id)
            .header("X-Naver-Client-Secret", &self.client_secret)
            .body(serde_urlencoded::to_string(data).unwrap())
            .send()
            .unwrap()
            .json()
            .unwrap();
        println!("{:?}", res);
        Ok(TranslationOutput {
            text: res.message.result.translated_text,
            lang: Language::from_str(&res.message.result.src_lang_type)?,
        })
    }

    fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> std::result::Result<TranslationVecOutput, Error> {
        let v = self.translate(client, &query.join("\n"), from, to)?;
        Ok(TranslationVecOutput {
            text: v.text.split('\n').map(|v| v.to_string()).collect(),
            lang: v.lang,
        })
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Result {
    #[serde(rename = "srcLangType")]
    pub src_lang_type: String,
    #[serde(rename = "tarLangType")]
    pub tar_lang_type: String,
    #[serde(rename = "translatedText")]
    pub translated_text: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Message {
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "@service")]
    pub _service: String,
    #[serde(rename = "@version")]
    pub _version: String,
    pub result: Result,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct PapagoResponse {
    pub message: Message,
}

#[derive(Serialize)]
struct PapagoRequest {
    source: String,
    target: String,
    text: String,
}

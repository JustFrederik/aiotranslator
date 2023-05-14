use async_trait::async_trait;
use std::collections::HashMap;
use std::str::FromStr;

use chrono::Local;
use reqwest::Client;
use serde::Deserialize;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorNoContext,
};

//https://docs.rs/crate/youdao/0.3.0/source/src/lib.rs
pub struct YouDaoTranslator {
    app_key: String,
    app_secret: String,
}
#[async_trait]
#[allow(dead_code)]
impl TranslatorNoContext for YouDaoTranslator {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        if query.is_empty() {
            return Err(Error::new_option("Empty query"));
        }

        let params = self.build_text_translate_form(
            query,
            from.map(|v| v.to_youdao_str())
                .unwrap_or_else(|| Ok("auto".to_string()))?,
            to.to_youdao_str()?,
        );
        let resp: TextTranslateResult = client
            .post("https://openapi.youdao.com/api")
            .form(&params)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        Ok(TranslationOutput {
            text: resp.translation.unwrap().join("\n"),
            lang: Language::from_str(&resp.l)?,
        })
    }

    async fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let v = self.translate(client, &query.join("\n"), from, to).await?;
        Ok(TranslationVecOutput {
            text: v.text.split('\n').map(|v| v.to_string()).collect(),
            lang: v.lang,
        })
    }
}
#[allow(dead_code)]
impl YouDaoTranslator {
    pub fn new(app_key: String, app_secret: String) -> YouDaoTranslator {
        Self {
            app_key,
            app_secret,
        }
    }

    fn build_text_translate_form(
        &self,
        q: &str,
        from: String,
        to: String,
    ) -> HashMap<String, String> {
        let salt = Local::now().timestamp_millis().to_string();
        let curtime = Local::now().timestamp().to_string();
        let mut params = HashMap::new();
        params.insert("q".to_string(), q.to_string());
        params.insert("from".to_string(), from);
        params.insert("to".to_string(), to);
        params.insert("appKey".to_string(), self.app_key.clone());
        params.insert("salt".to_string(), salt.to_owned());
        params.insert("signType".to_string(), "v3".to_string());
        params.insert("curtime".to_string(), curtime.to_owned());
        let mut sign_str = String::new();
        sign_str.push_str(&self.app_key);
        sign_str.push_str(&truncate(q));
        sign_str.push_str(&salt);
        sign_str.push_str(&curtime);
        sign_str.push_str(&self.app_secret);

        let sign = sha256::digest(sign_str);
        params.insert("sign".to_string(), sign);

        params
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
#[allow(dead_code)]
pub struct TextTranslateResult {
    pub return_phrase: Option<Vec<String>>,
    pub query: Option<String>,
    pub error_code: String,
    pub l: String,
    pub t_speak_url: Option<String>,
    pub web: Option<Vec<Web>>,
    pub request_id: Option<String>,
    pub translation: Option<Vec<String>>,
    pub dict: Option<Dict>,
    pub webdict: Option<Dict>,
    pub basic: Option<Basic>,
    pub speak_url: Option<String>,
    pub is_word: Option<bool>,
}

fn truncate(q: &str) -> String {
    if q.is_empty() {
        return "".to_string();
    }
    let chars: Vec<char> = q.chars().collect();
    let len = chars.len();
    if len <= 20 {
        q.to_owned()
    } else {
        let mut result = String::new();
        result.push_str(&String::from_iter(&chars[0..10]));
        result.push_str(&len.to_string());
        result.push_str(&String::from_iter(&chars[len - 10..len]));

        result
    }
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Web {
    pub key: String,
    pub value: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Dict {
    pub url: String,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Basic {
    #[serde(rename = "exam_type")]
    pub exam_type: Option<Vec<String>>,
    pub phonetic: Option<String>,
    #[serde(rename = "us-phonetic")]
    pub us_phonetic: Option<String>,
    #[serde(rename = "uk-phonetic")]
    pub uk_phonetic: Option<String>,
    pub wfs: Option<Vec<Wfs>>,
    #[serde(rename = "uk-speech")]
    pub uk_speech: Option<String>,
    pub explains: Vec<String>,
    #[serde(rename = "us-speech")]
    pub us_speech: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Wfs {
    pub wf: Wf,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Wf {
    pub name: String,
    pub value: String,
}

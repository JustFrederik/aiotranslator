use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use chrono::Local;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorNoContext,
};

//https://docs.rs/crate/youdao/0.3.0/source/src/lib.rs
pub struct YouDaoApiTranslator {
    app_key: String,
    app_secret: String,
}
#[async_trait]
#[allow(dead_code)]
impl TranslatorNoContext for YouDaoApiTranslator {
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
        let resp: TranslationResponse = client
            .post("https://openapi.youdao.com/api")
            .form(&params)
            .send()
            .await
            .map_err(Error::fetch)?
            .json()
            .await
            .map_err(Error::fetch)?;
        Ok(TranslationOutput {
            text: resp.translation.join("\n"),
            lang: Language::from_str(
                resp.l
                    .split('2')
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap_or(&"unknown"),
            )
            .unwrap_or(Language::Unknown),
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
impl YouDaoApiTranslator {
    pub fn new(app_key: &str, app_secret: &str) -> Self {
        Self {
            app_key: app_key.to_string(),
            app_secret: app_secret.to_string(),
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

#[derive(Deserialize)]
#[allow(dead_code)]
struct TranslationResponse {
    #[serde(rename = "tSpeakUrl")]
    pub t_speak_url: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub query: String,
    #[serde(rename = "isDomainSupport")]
    pub is_domain_support: bool,
    pub translation: Vec<String>,
    #[serde(rename = "mTerminalDict")]
    pub m_terminal_dict: Value,
    #[serde(rename = "errorCode")]
    pub error_code: String,
    pub dict: Value,
    pub webdict: Value,
    pub l: String,
    #[serde(rename = "isWord")]
    pub is_word: bool,
    #[serde(rename = "speakUrl")]
    pub speak_url: String,
}

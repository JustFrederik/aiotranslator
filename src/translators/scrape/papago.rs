use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use hmac::{Hmac, Mac};
use md5::Md5;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT};
use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::helpers::option_error;
use crate::translators::tokens::Tokens;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorLanguages, TranslatorNoContext,
};

pub struct PapagoTranslator {
    host: String,
}

#[cfg(feature = "fetch_languages")]
impl TranslatorLanguages for PapagoTranslator {
    fn get_languages(client: &Client, _: &Tokens) -> Result<Vec<String>, Error> {
        let se = Self::new();
        let lang_html = se.get_lang_html(client)?;
        let lang_re =
            Regex::new(r#"=\{ALL:(.*?)}"#).map_err(|e| Error::new("Invalid regex pattern", e))?;
        let lang_str = lang_re
            .captures(&lang_html)
            .ok_or_else(|| Error::new_option("Failed to find capture"))?[1]
            .to_string()
            .to_lowercase()
            .replace("zh-cn", "zh-CN")
            .replace("zh-tw", "zh-TW")
            .replace('\"', "");
        let lang_re2 = Regex::new(r#","(.*?)":|,(.*?):"#)
            .map_err(|e| Error::new("Invalid regex pattern", e))?;
        let mut lang_list: Vec<String> = lang_re2
            .find_iter(&lang_str)
            .map(|m| m.as_str().trim_matches(|c| c == ',' || c == ':').to_owned())
            .filter(|x| x != "auto")
            .collect();
        lang_list.sort();
        lang_list.dedup();
        Ok(lang_list)
    }
}

impl TranslatorNoContext for PapagoTranslator {
    fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        let url = format!("{}/apis/n2mt/translate", self.host);
        let auth_key = self.get_auth_key(client)?;
        let device_id = uuid::Uuid::new_v4().to_string();
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::new("System time error", e))?;
        let timestamp =
            since_epoch.as_secs() * 1000 + u64::from(since_epoch.subsec_nanos()) / 1_000_000;
        let auth = self.get_auth_ppg(&url, &auth_key, &device_id, timestamp)?;

        let data = TranslationRequest {
            device_id,
            text: query.to_string(),
            source: option_error(from.map(|v| v.to_papago_str()))?
                .unwrap_or_else(|| "auto".to_string()),
            target: to.to_papago_str()?,
            locale: "en".to_string(),
            dict: true,
            dict_display: 30,
            honorific: false,
            instant: false,
            paging: false,
        };

        let data =
            serde_urlencoded::to_string(data).map_err(|v| Error::new("Failed to serialize", v))?;

        let res: PapagoResponse = client
            .post(url)
            .header(ORIGIN, &self.host)
            .header(REFERER, &self.host)
            .header(AUTHORIZATION, auth)
            .header("timestamp", timestamp)
            .header("X-Requested-With", "XMLHttpRequest")
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header(USER_AGENT, "Mozilla/5.0")
            .header("device-type", "pc")
            .header("x-apigw-partnerid", "papago")
            .body(data)
            .send()
            .map_err(|e| Error::new("Failed to get response text", e))?
            .json()
            .map_err(|e| Error::new("Failed to deserialze", e))?;
        Ok(TranslationOutput {
            text: res.translated_text,
            lang: Language::from_str(&res.src_lang_type)?,
        })
    }

    fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let v = self.translate(client, &query.join("\n"), from, to)?;
        Ok(TranslationVecOutput {
            text: v.text.split('\n').map(|v| v.to_string()).collect(),
            lang: v.lang,
        })
    }
}

impl Default for PapagoTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl PapagoTranslator {
    pub fn new() -> Self {
        Self {
            host: "https://papago.naver.com".to_string(),
        }
    }

    pub fn get_auth_key(&self, client: &Client) -> Result<String, Error> {
        let lang_html = self.get_lang_html(client)?;
        let auth_key_regex =
            Regex::new(r#"AUTH_KEY:"(.*?)""#).map_err(|e| Error::new("Wrong regex pattern", e))?;
        Ok(auth_key_regex
            .captures(&lang_html)
            .ok_or_else(|| Error::new_option("Failed capture"))?[1]
            .to_string())
    }

    pub fn get_lang_html(&self, client: &Client) -> Result<String, Error> {
        let data = client
            .get(&self.host)
            .send()
            .map_err(|e| Error::new("Failed to get response", e))?
            .text()
            .map_err(|e| Error::new("Failed to get response text", e))?;
        let url_path_regex = Regex::new(r"/home\.(.*?)\.chunk\.js")
            .map_err(|e| Error::new("Wrong regex pattern", e))?;
        let url_path = url_path_regex
            .captures(&data)
            .ok_or_else(|| Error::new_option("Failed capture"))?[0]
            .to_string();
        let lang_detect_url = format!("{}{}", self.host, url_path);
        let lang_html = client
            .get(lang_detect_url)
            .send()
            .map_err(|e| Error::new("Failed to get response", e))?
            .text()
            .map_err(|e| Error::new("Failed to get response text", e))?;
        Ok(lang_html)
    }

    fn get_auth_ppg(
        &self,
        url: &str,
        auth_key: &str,
        device_id: &str,
        time_stamp: u64,
    ) -> Result<String, Error> {
        let value = format!(
            "{}\n{}\n{}",
            device_id,
            url.split('?')
                .next()
                .ok_or_else(|| Error::new_option("Invalid url"))?,
            time_stamp
        );
        let mut mac = Hmac::<Md5>::new_from_slice(auth_key.as_bytes())
            .map_err(|e| Error::new("The key has a invalid length", e))?;
        mac.update(value.as_bytes());
        let result = mac.finalize().into_bytes();
        Ok(format!(
            "PPG {}:{}",
            device_id,
            base64::engine::general_purpose::STANDARD.encode(result)
        ))
    }
}

#[derive(Serialize)]
#[allow(dead_code)]
struct TranslationRequest {
    #[serde(rename = "deviceId")]
    device_id: String,
    text: String,
    source: String,
    target: String,
    locale: String,
    dict: bool,
    #[serde(rename = "dictDisplay")]
    dict_display: i32,
    honorific: bool,
    instant: bool,
    paging: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct PapagoResponse {
    pub delay: i64,
    #[serde(rename = "delaySmt")]
    pub delay_smt: i64,
    #[serde(rename = "srcLangType")]
    pub src_lang_type: String,
    #[serde(rename = "tarLangType")]
    pub tar_lang_type: String,
    #[serde(rename = "translatedText")]
    pub translated_text: String,
    #[serde(rename = "engineType")]
    pub engine_type: String,
}

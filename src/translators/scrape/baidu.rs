use std::str::FromStr;

use async_trait::async_trait;
#[cfg(feature = "fetch_languages")]
use regex::Regex;
use reqwest::header::{ORIGIN, REFERER};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::helpers::option_error;
use crate::translators::tokens::Tokens;
use crate::translators::translator_structrue::{
    TranslationOutput, TranslationVecOutput, TranslatorLanguages, TranslatorNoContext,
};

pub struct BaiduTranslator {
    /// base url
    host: String,
    /// api url
    api_host: String,
}

impl Default for BaiduTranslator {
    /// default is new
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
#[cfg(feature = "fetch_languages")]
impl TranslatorLanguages for BaiduTranslator {
    /// returns all available languages
    /// step1 => regex('https://fanyi-cdn.cdn.bcebos.com/webStatic/translation/js/index.(.*?).js').search(host_html).group()
    /// step1.5 => curl(step1)
    /// step2 => regex('exports={auto:(.*?)}}}},').search(js_html).group()[8:-3]
    /// step3 => regex('(\w+):{zhName:').findall(step2)
    /// partially generated by chatgpt
    async fn get_languages(client: &Client, _: &Tokens) -> Result<Vec<String>, Error> {
        let s = Self::new();
        let data = client
            .get(&s.host)
            .send()
            .await
            .map_err(|e| Error::new(format!("Failed to send request to {}", s.host), e))?
            .text()
            .await
            .map_err(|e| Error::new("Failed to get response text", e))?;
        let get_lang_url_regex =
            Regex::new(r"https://fanyi-cdn.cdn.bcebos.com/webStatic/translation/js/index.(.*?).js")
                .map_err(|e| Error::new("Failed to create regex", e))?;
        let get_lang_url = get_lang_url_regex
            .find(&data)
            .ok_or_else(|| Error::new_option("failed to find regex"))?
            .as_str()
            .to_string();
        let js_html = client
            .get(&get_lang_url)
            .send()
            .await
            .map_err(|e| Error::new(format!("Failed to send request to {}", get_lang_url), e))?
            .text()
            .await
            .map_err(|e| Error::new("Failed to get response text", e))?;
        let re_lang_str = Regex::new(r"exports=\{auto:(.*?)}}}},")
            .map_err(|e| Error::new("Failed to create regex", e))?;
        let lang_str = re_lang_str
            .captures(&js_html)
            .ok_or_else(|| Error::new_option("Failed captures"))?
            .get(1)
            .ok_or_else(|| Error::new_option("Failed to get 1"))?
            .as_str();
        let lang_str = lang_str[8..lang_str.len() - 3].to_string();

        let re_lang_list =
            Regex::new(r"(\w+):\{zhName:").map_err(|e| Error::new("Failed to create regex", e))?;
        let lang_list = re_lang_list
            .find_iter(&lang_str)
            .map(|mat| mat.as_str().trim_end_matches(":{zhName:").to_string())
            .collect::<Vec<String>>();
        let mut lang_list = lang_list.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        lang_list.sort();
        lang_list.dedup();
        Ok(lang_list.iter().map(|s| s.to_string()).collect())
    }
}

#[async_trait]
impl TranslatorNoContext for BaiduTranslator {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        self.translate_vec(client, &[query.to_string()], from, to)
            .await
            .map(|v| TranslationOutput {
                lang: v.lang,
                text: v.text.join("\n"),
            })
    }

    async fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let query = TranslateResult {
            from: option_error(from.map(|v| v.to_baidu_str()))?
                .unwrap_or_else(|| "auto".to_string()),
            to: to.to_baidu_str()?,
            source: String::from("txt"),
            query: query.join("\n"),
        };
        let data = serde_urlencoded::to_string(&query)
            .map_err(|e| Error::new("Failed to encode query", e))?;

        let v: BaiduResponse = client
            .post(&self.api_host)
            .header(REFERER, &self.host)
            .header(ORIGIN, &self.host)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("X-Requested-With", "XMLHttpRequest")
            .body(data)
            .send()
            .await
            .map_err(|e| Error::new(format!("Failed to send request to {}", self.api_host), e))?
            .json()
            .await
            .map_err(|e| Error::new("Failed to get response text", e))?;
        Ok(TranslationVecOutput {
            text: v.data.iter().map(|v| v.dst.clone()).collect(),
            lang: Language::from_str(&v.from)?,
        })
    }
}

impl BaiduTranslator {
    /// sets urls
    pub fn new() -> Self {
        Self {
            host: "https://fanyi.baidu.com".to_string(),
            api_host: "https://fanyi.baidu.com/transapi".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
struct TranslateResult {
    /// source language
    /// default = auto
    from: String,
    /// target language
    to: String,
    /// default value txt
    source: String,
    /// text to translate
    query: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Data {
    /// translation
    pub dst: String,
    /// idk
    #[serde(rename = "prefixWrap")]
    pub prefix_wrap: i64,
    /// idk what this stands for, probably more details about translation
    pub result: Value,
    /// original text
    pub src: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct BaiduResponse {
    /// translated content
    pub data: Vec<Data>,
    /// detected language of source
    pub from: String,
    /// server response code
    pub status: i64,
    /// target language
    pub to: String,
    /// idk what this stands for
    #[serde(rename = "type")]
    pub r#type: i64,
}

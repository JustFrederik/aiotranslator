use std::collections::HashSet;

use async_trait::async_trait;
use reqwest::header::{ORIGIN, REFERER};
use reqwest::Client;
#[cfg(feature = "fetch_languages")]
use select::document::Document;
#[cfg(feature = "fetch_languages")]
use select::predicate::Name;
use serde::Serialize;
use serde_json::Value;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::tokens::Tokens;
use crate::translators::translator_structrue::{
    TranslationOutput, TranslationVecOutput, TranslatorLanguages, TranslatorNoContext,
};

pub struct YoudaoTranslator {
    host: String,
    home: String,
}

impl Default for YoudaoTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
#[cfg(feature = "fetch_languages")]
impl TranslatorLanguages for YoudaoTranslator {
    async fn get_languages(client: &Client, _: &Tokens) -> Result<Vec<String>, Error> {
        let se = Self::new();
        let data = client
            .get(format!("{}{}", se.host, se.home))
            .send()
            .await
            .map_err(|e| Error::new("Failed request", e))?
            .text()
            .await
            .map_err(|e| Error::new("Failed request", e))?;
        let mut lang_list: Vec<String> = Vec::new();
        let document = Document::from_read(data.as_bytes())
            .map_err(|e| Error::new("Failed to parse html", e))?;
        for link in document.find(Name("a")) {
            if let Some(val) = link.attr("val") {
                lang_list.push(val.to_owned());
            }
        }
        let mut seen = HashSet::new();
        seen.insert("AUTO".to_string());
        seen.insert("SPACE".to_string());
        let mut item: Vec<_> = lang_list
            .iter()
            .flat_map(|lang| {
                lang.to_string()
                    .split('2')
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
            })
            .collect();
        item.retain(|x| seen.insert(x.to_string()));
        Ok(item)
    }
}

impl YoudaoTranslator {
    pub fn new() -> Self {
        Self {
            host: String::from("https://ai.youdao.com"),
            home: String::from("/product-fanyi-text.s"),
        }
    }
}

#[async_trait]
impl TranslatorNoContext for YoudaoTranslator {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        _from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        to.to_youdao_str()?;
        let data = YoudaoRequest {
            query: query.to_string(),
            from: String::from("Auto"),
            to: String::from("Auto"),
        };
        let data =
            serde_urlencoded::to_string(&data).map_err(|v| Error::new("Failed to serialize", v))?;
        let resp = client
            .post("https://aidemo.youdao.com/trans")
            .header(ORIGIN, &self.host)
            .header(REFERER, format!("{}{}", self.host, self.home))
            .header("X-Requested-With", "XMLHttpRequest")
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .body(data)
            .send()
            .await
            .map_err(|e| Error::new("Failed request", e))?
            .text()
            .await
            .map_err(|e| Error::new("Failed request", e))?;
        let v: Value =
            serde_json::from_str(&resp).map_err(|e| Error::new("Failed to parse json", e))?;
        let res = v["translation"]
            .as_array()
            .ok_or_else(|| Error::new_option("invalid response"))?;
        let r = res
            .iter()
            .map(|r| {
                r.as_str()
                    .ok_or_else(|| Error::new_option("Value is empty"))
                    .map(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(TranslationOutput {
            text: r.join("._._._."),
            lang: Language::Unknown,
        })
    }

    async fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let v = self
            .translate(client, &query.join("._._._."), from, to)
            .await?;
        Ok(TranslationVecOutput {
            text: v.text.split("._._._.").map(|v| v.to_string()).collect(),
            lang: Language::Unknown,
        })
    }
}

#[derive(Serialize)]
struct YoudaoRequest {
    #[serde(rename = "q")]
    query: String,
    from: String,
    to: String,
}

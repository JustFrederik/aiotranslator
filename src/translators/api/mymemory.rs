use async_trait::async_trait;
use reqwest::Client;
#[cfg(feature = "fetch_languages")]
use select::document::Document;
#[cfg(feature = "fetch_languages")]
use select::predicate::{Attr, Name};
use serde_json::Value;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::helpers::input_limit_checker;
use crate::translators::tokens::Tokens;
use crate::translators::translator_structrue::{
    TranslationOutput, TranslationVecOutput, TranslatorLanguages, TranslatorNoContext,
};

pub struct MyMemoryTranslator {
    /// how long the text to translate can be
    input_limit: u32,
    /// host url
    host: String,
}

/// default value
impl Default for MyMemoryTranslator {
    /// new is default
    fn default() -> Self {
        MyMemoryTranslator::new()
    }
}

#[async_trait]
#[cfg(feature = "fetch_languages")]
impl TranslatorNoContext for MyMemoryTranslator {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        input_limit_checker(query, self.input_limit)?;
        let url = format!(
            "{}?q={}&langpair={}|{}",
            self.host,
            query,
            from.ok_or_else(|| Error::new_option("Requires from"))?
                .to_mymemory_str()?,
            to.to_mymemory_str()?
        );
        let headers = {
            let mut map = reqwest::header::HeaderMap::new();
            map.insert(
                "Referer",
                "https://mymemory.translated.net"
                    .parse()
                    .map_err(|e| Error::new("Failed to parse referer", e))?,
            );
            map
        };
        let response = client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::new(format!("Failed get request to {}", url), e))?;
        if !response.status().is_success() {
            return Err(Error::new_option(format!(
                "Request failed: {}",
                response.status()
            )));
        }
        let resp: Value = response
            .json()
            .await
            .map_err(|e| Error::new("Failed to deserialize", e))?;
        let mut text = resp["responseData"]["translatedText"].to_string();
        if text == "null" {
            return Err(Error::new_option("No translation found"));
        }
        if text.starts_with('"') && text.ends_with('"') {
            text = text[1..text.len() - 1].to_string();
        }
        Ok(TranslationOutput {
            text,
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
            .translate(client, &query.join("_._._"), from, to)
            .await?
            .text
            .split("_._._")
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        Ok(TranslationVecOutput {
            text: v,
            lang: Language::Unknown,
        })
    }
}

#[async_trait]
impl TranslatorLanguages for MyMemoryTranslator {
    /// gets all languages
    /// xpath('//*[@id="select_source_mm"]/option/@value')[2:]
    /// partially generated by chatgpt
    async fn get_languages(client: &Client, _: &Tokens) -> Result<Vec<String>, Error> {
        let data = client
            .get("https://mymemory.translated.net")
            .send()
            .await
            .map_err(|v| Error::new("Failed get request to https://mymemory.translated.net", v))?;
        if !data.status().is_success() {
            return Err(Error::new_option(format!(
                "Request failed with status code {}",
                data.status()
            )));
        }
        let data = data
            .text()
            .await
            .map_err(|e| Error::new("Failed to extract text from response", e))?;
        let et = Document::from_read(data.as_bytes())
            .map_err(|e| Error::new("Failed to parse html", e))?;
        let lang_list = et.find(Attr("id", "select_source_mm")).next().map(|n| {
            n.find(Name("option"))
                .filter_map(|n| n.attr("value"))
                .skip(2)
        });
        if lang_list.is_none() {
            return Err(Error::new_option("Failed to find language list"));
        }
        let mut lang_list = lang_list
            .ok_or_else(|| Error::new_option("Checked before"))?
            .into_iter()
            .collect::<Vec<&str>>();
        lang_list.sort();
        lang_list.dedup();
        Ok(lang_list.iter().map(|l| l.to_string()).collect())
    }
}

impl MyMemoryTranslator {
    pub fn new() -> Self {
        MyMemoryTranslator {
            input_limit: 500,
            host: "https://api.mymemory.translated.net/get".to_string(),
        }
    }
}

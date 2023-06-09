use std::str::FromStr;

use reqwest::blocking::Client;
use reqwest::header::{CONTENT_TYPE, REFERER};
#[cfg(feature = "fetch_languages")]
use select::document::Document;
#[cfg(feature = "fetch_languages")]
use select::predicate::{Attr, Name, Predicate};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::helpers::{input_limit_checker, option_error};
use crate::translators::tokens::Tokens;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorLanguages, TranslatorNoContext,
};

#[derive(Debug)]
pub struct BingTranslator {
    /// website url
    host: String,
    /// api url
    api_host: String,
    /// extracted key
    key: i64,
    /// extracted token
    token: String,
    /// extracted ig
    ig: String,
    /// iid value (not sure what it does)
    /// translator.5028 or translator.5027 doesnt matter for the result
    iid: String,
    /// how long the text to translate can be
    input_limit: u32,
}

#[cfg(feature = "fetch_languages")]
impl TranslatorNoContext for BingTranslator {
    fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        input_limit_checker(query, self.input_limit)?;
        let api_url = format!(
            "{}?isVertical=1&&IG={}&IID={}",
            self.api_host, self.ig, self.iid
        );
        let data = TranslationQuery {
            text: query.to_string(),
            from_language: option_error(from.map(|v| v.to_baidu_str()))?
                .unwrap_or_else(|| "auto-detect".to_string()),
            to: to.to_bing_str()?,
            try_fetching_gender_debiased_translations: "true".to_string(),
            key: self.key,
            token: self.token.clone(),
        };
        let data =
            serde_urlencoded::to_string(data).map_err(|v| Error::new("Failed to serialize", v))?;

        let response = client
            .post(&api_url)
            .header(REFERER, &self.host)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(data)
            .send()
            .map_err(|v| Error::new(format!("Failed post request to {}", api_url), v))?;
        if !response.status().is_success() {
            return Err(Error::new_option(format!(
                "Request failed with status code {}",
                response.status()
            )));
        }
        //FIXME: why isnt Vec<BingResponse> working?
        let mut json: Vec<Value> = response
            .json()
            .map_err(|v| Error::new("Failed to deserialze", v))?;
        if json.is_empty() {
            return Err(Error::new_option("No translation found"));
        }
        let first_value = json.remove(0);
        let temp = serde_json::from_value::<BingResponse>(first_value)
            .map_err(|v| Error::new("Failed to deserialize", v))?;
        Ok(TranslationOutput {
            text: temp
                .translations
                .first()
                .as_ref()
                .ok_or_else(|| Error::new_option("No translation found"))?
                .text
                .to_string(),
            lang: Language::from_str(&temp.detected_language.language)?,
        })
    }

    fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let query = query.join("\n");
        let trans = self.translate(client, &query, from, to)?;
        Ok(TranslationVecOutput {
            text: trans.text.split('\n').map(|v| v.to_string()).collect(),
            lang: trans.lang,
        })
    }
}

impl TranslatorLanguages for BingTranslator {
    /// returns all available languages
    /// xpath('//*[@id="tta_srcsl"]/option/@value') or xpath('//*[@id="t_srcAllLang"]/option/@value')
    /// partially generated by chatgpt
    fn get_languages(client: &Client, _: &Tokens) -> Result<Vec<String>, Error> {
        let data = client
            .get("https://www.bing.com/Translator")
            .send()
            .map_err(|v| Error::new("Failed get request to https://www.bing.com/Translator", v))?;
        if !data.status().is_success() {
            return Err(Error::new_option(format!(
                "Request failed with status code {}",
                data.status()
            )));
        }
        let data = data
            .text()
            .map_err(|e| Error::new("Failed to extract text from response", e))?;
        let et = Document::from_read(data.as_bytes())
            .map_err(|e| Error::new("Failed to parse html", e))?;
        let temp = et
            .find(Attr("id", "tta_srcsl").descendant(Name("option")))
            .map(|n| {
                n.attr("value")
                    .ok_or_else(|| Error::new_option("Couldnt get languages because value is None"))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let mut lang_list = temp
            .iter()
            .map(|v| v.to_string())
            .chain(
                et.find(Attr("id", "t_srcAllLang").descendant(Name("option")))
                    .map(|n| {
                        n.attr("value")
                            .ok_or_else(|| {
                                Error::new_option("Couldnt get languages because value is None")
                            })
                            .map(|v| v.to_string())
                    })
                    .collect::<Result<Vec<_>, Error>>()?,
            )
            .collect::<Vec<_>>();
        lang_list.sort();
        lang_list.dedup();
        Ok(lang_list)
    }
}

impl BingTranslator {
    /// initializes values
    pub fn new(client: &Client) -> Result<Self, Error> {
        //TODO: other urls
        //let host = "https://cn.bing.com/Translator";
        let host = "https://www.bing.com/Translator".to_string();
        let api_host = host.replace("Translator", "ttranslatev3");
        let data = client
            .get(&host)
            .send()
            .map_err(|v| Error::new(format!("Failed get request to {}", host), v))?;
        if !data.status().is_success() {
            return Err(Error::new_option(format!(
                "Request failed with status code {}",
                data.status()
            )));
        }
        let data = data
            .text()
            .map_err(|e| Error::new("Failed to extract text from response", e))?;
        let tk = get_tk(&data)?;
        let ig = get_ig(&data)?;
        Ok(BingTranslator {
            host,
            api_host,
            key: tk.0,
            token: tk.1,
            ig,
            iid: "translator.5028".to_string(),
            input_limit: 1000,
        })
    }
}

/// extracts token and key
fn get_tk(htlm_code: &str) -> Result<(i64, String), Error> {
    let re = regex::Regex::new(r#"var params_AbusePreventionHelper = (.*?);"#)
        .map_err(|e| Error::new("Failed to create regex", e))?;
    let captures = re
        .captures(htlm_code)
        .ok_or_else(|| Error::new_option("Capture group not found"))?;
    let result_str = captures
        .get(1)
        .ok_or_else(|| Error::new_option("No match for capture group"))?
        .as_str()
        .to_string();
    let tuple = parse_tuple(&result_str)?;
    Ok((tuple.0, tuple.1))
}

/// extracts ig
fn get_ig(html: &str) -> Result<String, Error> {
    let re = regex::Regex::new(r#"IG:"(.*?)""#)
        .map_err(|e| Error::new("Failed to create regex pattern", e))?;
    let ig = re
        .captures(html)
        .ok_or_else(|| Error::new_option("Capture group not found"))?
        .get(1)
        .ok_or_else(|| Error::new_option("Failed to get capture group"))?
        .as_str();
    Ok(ig.to_string())
}

/// Convert string to the tupple (i64, String, i64) which contains key and token
fn parse_tuple(s: &str) -> Result<(i64, String, i64), Error> {
    let v: Vec<Value> =
        serde_json::from_str(s).map_err(|e| Error::new("Failed to deserialize into Vector", e))?;
    if v.len() != 3 {
        return Err(Error::new_option("Expected tuple of length 3"));
    }
    let val1 = v[0]
        .as_i64()
        .ok_or_else(|| Error::new_option("Expected integer in element 1"))?;
    let val2 = v[1]
        .as_str()
        .ok_or_else(|| Error::new_option("Expected string in element 2"))?
        .to_string();
    let val3 = v[2]
        .as_i64()
        .ok_or_else(|| Error::new_option("Expected integer in element 3"))?;
    Ok((val1, val2, val3))
}

#[derive(Debug, Serialize)]
/// Request body for the Bing translation
struct TranslationQuery {
    /// The text that will be translated
    text: String,
    /// source language
    /// could be auto-detect
    #[serde(rename = "fromLang")]
    from_language: String,
    /// target language
    to: String,
    #[serde(rename = "tryFetchingGenderDebiasedTranslations")]
    try_fetching_gender_debiased_translations: String,
    /// key fetched from the html
    key: i64,
    /// token fetched from the html
    token: String,
}

#[derive(Deserialize)]
/// Response from the Bing translation
pub struct SentLen {
    /// The length of the source sentences
    #[serde(alias = "srcSentLen")]
    pub src_sent_len: Vec<i64>,
    /// The length of the translated sentences
    #[serde(alias = "transSentLen")]
    pub trans_sent_len: Vec<i64>,
}

#[derive(Deserialize)]
/// Response from the Bing translation
pub struct Translation {
    /// The translated text
    pub text: String,
    /// The target language
    pub to: String,
    /// The length of the sentences
    #[serde(alias = "sentLen")]
    pub sent_len: SentLen,
}

#[derive(Deserialize)]
/// Response from the Bing translation
pub struct DetectedLanguage {
    /// The detected language
    pub language: String,
    /// The probability of the detected language
    pub score: f64,
}

#[derive(Deserialize)]
/// Response from the Bing translation
pub struct BingResponse {
    /// Detected language data
    #[serde(rename = "detectedLanguage")]
    pub detected_language: DetectedLanguage,
    /// List of translations
    pub translations: Vec<Translation>,
}

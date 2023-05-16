//https://docs.rs/crate/translation-api-cn/latest/source/src/baidu.rs

use crate::error::Error;
use crate::languages::Language;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorNoContext,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

#[allow(dead_code)]
pub struct BaiduApiTranslator {
    url: String,
    app_id: String,
    key: String,
}

#[async_trait]
impl TranslatorNoContext for BaiduApiTranslator {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        let form = Form::new(
            &self.app_id,
            query,
            "0",
            &self.key,
            &from
                .map(|v| v.to_baidu_str())
                .unwrap_or_else(|| Ok("auto".to_string()))?,
            &to.to_baidu_str()?,
        );
        let resp: Response = client
            .post(&self.url)
            .form(&form)
            .send()
            .await
            .map_err(Error::fetch)?
            .json()
            .await
            .map_err(Error::fetch)?;
        let resp = match resp {
            Response::Ok(v) => v,
            Response::Err(v) => return Err(Error::baidu_error(v)),
        };
        Ok(TranslationOutput {
            text: resp
                .trans_result
                .iter()
                .map(|v| v.dst.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            lang: Language::from_str(&resp.from).unwrap_or(Language::Unknown),
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
impl BaiduApiTranslator {
    pub fn new(app_id: &str, key: &str) -> Self {
        Self {
            url: "https://fanyi-api.baidu.com/api/trans/vip/translate".to_string(),
            app_id: app_id.to_string(),
            key: key.to_string(),
        }
    }
}

/// The data submitted by the form
#[derive(Debug, Serialize)]
pub struct Form {
    pub q: String,
    pub from: String,
    pub to: String,
    pub appid: String,
    pub salt: String,
    pub sign: String,
}

impl Form {
    fn new(appid: &str, q: &str, salt: &str, key: &str, from: &str, to: &str) -> Self {
        let data = format!("{}{}{}{}", &appid, q, salt, key);
        let sign = format!("{:x}", md5_alt::compute(data));
        Self {
            q: q.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            appid: appid.to_string(),
            salt: salt.to_string(),
            sign,
        }
    }
}

/// Response information. Either return the translation result, or return an error message.
#[derive(Deserialize)]
#[serde(untagged)]
enum Response {
    Ok(TranslationResponse),
    Err(BaiduApiError),
}

/// Error handling / error code
#[derive(Debug, Clone, Deserialize)]
pub struct BaiduApiError {
    #[serde(rename = "error_code")]
    pub code: String,
    #[serde(rename = "error_msg")]
    pub msg: String,
    pub data: Option<Value>,
}

impl std::error::Error for BaiduApiError {}
impl std::fmt::Display for BaiduApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "Error code: `{}`\nError message: `{}`\nError meaning: {}\nThe above content is returned by Baidu translation API",
               self.code,
               self.msg,
               self.solution())
    }
}

impl BaiduApiError {
    ///Reference: [Error Code List](https://fanyi-api.baidu.com/doc/21)
    pub fn solution(&self) -> &str {
        match self.code.as_bytes() {
            b"52000" => "success",
            b"52001" => "Request timed out. \nSolution: Please try again.",
            b"52002" => "system error. \nSolution: Please try again.",
            b"52003" => "Unauthorized user. \nSolution: Please check whether the appid is correct or whether the service is enabled.",
            b"54000" => "The required parameter is empty. \nSolution: Please check whether to pass fewer parameters.",
            b"54001" => "Wrong signature. \nSolution: Please check your signature generation method.",
            b"54003" => "Frequency of access is limited. \nSolution: Please reduce your calling frequency, or switch to the premium version after authentication.",
            b"54004" => "Insufficient account balance. \nSolution: Please go to the management console to recharge the account.",
            b"54005" => "Long query requests are frequent. \nSolution: Please reduce the sending frequency of long queries and try again after 3s.",
            b"58000" => {
                "Client IP is illegal. \nSolution: Check whether the IP address filled in the personal information is correct, and you can go to Developer Information-Basic Information to modify it."
            }
            b"58001" => "Target language direction is not supported. \nSolution: Check if the target language is in the language list.",
            b"58002" => "The service is currently down. \nSolution: Please go to the management console to enable the service.",
            b"90107" => "The certification has not passed or is not valid. \nSolution: Please go to My Certification to check the certification progress.",
            _ => "unknown error",
        }
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Sentence {
    pub src: String,
    pub dst: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TranslationResponse {
    pub from: String,
    pub to: String,
    pub trans_result: Vec<Sentence>,
}

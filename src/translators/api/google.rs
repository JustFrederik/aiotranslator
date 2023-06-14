//TODO: implement
//url: https://translation.googleapis.com/language/translate/v2

use reqwest::blocking::Client;
use serde::Serialize;

#[allow(dead_code)]
#[derive(Serialize)]
struct GoogleRequest {
    /// Required The input text to translate. Provide an array of strings to translate multiple phrases. The maximum number of strings is 128.
    q: Vec<String>,
    /// Required The language to use for translation of the input text, set to one of the language codes listed in Language Support.
    target: String,
    /// The format of the source text, in either HTML (default) or plain-text. A value of html indicates HTML and a value of text indicates plain-text.
    format: String,
    /// The language of the source text, set to one of the language codes listed in Language Support. If the source language is not specified, the API will attempt to detect the source language automatically and return it within the response.
    source: Option<String>,
    /// The translation model. Cloud Translation - Basic offers only the nmt Neural Machine Translation (NMT) model.
    /// If the model is base, the request is translated by using the NMT model.
    model: Option<String>,
    /// A valid API key to handle requests for this API. If you are using OAuth 2.0 service account credentials (recommended), do not supply this parameter.
    key: String,
}

#[allow(dead_code)]
impl GoogleRequest {
    fn new(key: &str, from: Option<String>, to: String, queries: Vec<String>) -> Self {
        Self {
            q: queries,
            target: to,
            format: "text".to_string(), //html possible
            source: from,
            model: Some("base".to_string()),
            key: key.to_string(),
        }
    }

    pub async fn translate(&self, client: &Client) {
        let text = client
            .post("https://translation.googleapis.com/language/translate/v2")
            .json(self)
            .send()
            .unwrap()
            .text()
            .unwrap();
        println!("{}", text);
    }
}

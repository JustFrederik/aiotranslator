use edge_gpt::{ChatSession, ConversationStyle, CookieInFile};
use reqwest::Client;
use crate::error::Error;
use crate::languages::Language;
use crate::translators::{chatbot, ConversationStyleClone};
use crate::translators::context::{Context, get_gpt_context};
use crate::translators::helpers::input_limit_checker;
use crate::translators::translator_structrue::{TranslationOutput, TranslationVecOutput, TranslatorContext};
use async_trait::async_trait;
/// using https://github.com/acheong08/EdgeGPT
pub struct EdgeGpt {
    cookies: Vec<CookieInFile>,
    conversation_style: ConversationStyle,
    max_length: u32,
}

#[async_trait]
impl TranslatorContext for EdgeGpt {
    async fn translate(&self, _: &Client, query: &str, _: Option<Language>, to: &Language, context: &Vec<Context>) -> Result<TranslationOutput, Error> {
        unimplemented!()
    }

    async fn translate_vec(&self, _: &Client, query: &[String], _: Option<Language>, to: &Language, context: &Vec<Context>) -> Result<TranslationVecOutput, Error> {
        let con = get_gpt_context(context);
        let q_s = chatbot::generate_query(query, &to.to_name_str()?, con)?;
        let message = self.fetch(&q_s).await?;
        println!("{}", message);
        chatbot::process_result(message, query)
    }
}


impl EdgeGpt {
    pub async fn new(conversation_style_clone: ConversationStyleClone, cookies: &str) -> Result<Self, Error>{
        let conversation_style = match conversation_style_clone {
            ConversationStyleClone::Creative => ConversationStyle::Creative,
            ConversationStyleClone::Balanced => ConversationStyle::Balanced,
            ConversationStyleClone::Precise =>ConversationStyle::Precise,
        };
        let cookies: Vec<CookieInFile> = serde_json::from_str(cookies).map_err(|e| Error::new("Failed to deserialize cookies", e))?;
        Ok(Self {
            cookies,
            conversation_style,
            max_length: 2000
        })

    }


    pub async fn fetch(&self, question: &str) -> Result<String, Error>{
        input_limit_checker(question, self.max_length)?;
        let mut session = ChatSession::create(self.conversation_style, &self.cookies).await.map_err(|e| Error::new("Failed to create chat session", e))?;
        let response = session.send_message(question).await.map_err(|e| Error::new("Failed to send message", e))?;
        Ok(response.text)
    }
}

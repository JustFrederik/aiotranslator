use crate::error::Error;
use crate::languages::Language;
use crate::translators::context::{get_gpt_context, Context};
use crate::translators::helpers::input_limit_checker;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorContext,
};
use crate::translators::{chatbot, ConversationStyleClone};
use edge_gpt::{ChatSession, ConversationStyle, CookieInFile};
use futures::executor::block_on;
use reqwest::blocking::Client;
/// using https://github.com/acheong08/EdgeGPT
pub struct EdgeGpt {
    cookies: Vec<CookieInFile>,
    conversation_style: ConversationStyle,
    max_length: u32,
}

impl TranslatorContext for EdgeGpt {
    fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
        context: &[Context],
    ) -> Result<TranslationOutput, Error> {
        let v = self.translate_vec(client, &[query.to_string()], from, to, context)?;
        Ok(TranslationOutput {
            text: v.text.join("\n"),
            lang: v.lang,
        })
    }

    fn translate_vec(
        &self,
        _: &Client,
        query: &[String],
        _: Option<Language>,
        to: &Language,
        context: &[Context],
    ) -> Result<TranslationVecOutput, Error> {
        let con = get_gpt_context(context);
        let q_s = chatbot::generate_query(query, &to.to_name_str()?, con)?;
        let message = self.fetch(&q_s)?;
        println!("{}", message);
        chatbot::process_result(message, query)
    }
}

impl EdgeGpt {
    pub fn new(
        conversation_style_clone: &ConversationStyleClone,
        cookies: &str,
    ) -> Result<Self, Error> {
        let conversation_style = match conversation_style_clone {
            ConversationStyleClone::Creative => ConversationStyle::Creative,
            ConversationStyleClone::Balanced => ConversationStyle::Balanced,
            ConversationStyleClone::Precise => ConversationStyle::Precise,
        };
        let cookies: Vec<CookieInFile> = serde_json::from_str(cookies)
            .map_err(|e| Error::new("Failed to deserialize cookies", e))?;
        Ok(Self {
            cookies,
            conversation_style,
            max_length: 2000,
        })
    }

    pub fn fetch(&self, question: &str) -> Result<String, Error> {
        input_limit_checker(question, self.max_length)?;
        let response = block_on(async {
            let mut session = ChatSession::create(self.conversation_style, &self.cookies)
                .await
                .map_err(|e| Error::new("Failed to create chat session", e))?;
            session
                .send_message(question)
                .await
                .map_err(|e| Error::new("Failed to send message", e))
        })?;

        Ok(response.text)
    }
}

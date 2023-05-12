use std::collections::HashMap;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::translator_structure::TranslationVecOutput;

/// extract translations from response
pub fn process_result(s: String, queries: &[String]) -> Result<TranslationVecOutput, Error> {
    let mut translated = HashMap::new();
    for x in s.split('\n').filter(|v| v.contains(": ")) {
        let splitter = x.split_once(": ");
        let value = match splitter {
            Some(v) => Ok(v),
            None => Err(Error::new_option("syntax error => missing: ")),
        }?;
        translated.insert(
            value
                .0
                .parse::<usize>()
                .map_err(|e| Error::new("Failed to parse numbers for each prompt", e))?
                - 1,
            value.1,
        );
    }
    if translated.len() != queries.len() {
        return Err(Error::new_option(format!(
            "Expected {} translations, got {}",
            queries.len(),
            translated.len()
        )));
    }
    let mut result = vec![];
    for x in queries.iter().enumerate() {
        match translated.get(&x.0) {
            Some(v) => result.push(v.to_string()),
            None => return Err(Error::new_option(format!("No translation for {}", x.1))),
        };
    }
    Ok(TranslationVecOutput {
        text: result,
        lang: Language::Unknown,
    })
}

/// Generate text for chatgpt
pub fn generate_query(
    queries: &[String],
    target: &str,
    context: Option<&str>,
) -> Result<String, Error> {
    let mut query = format!("{}Can you translate these sentences to {}? Please keep the numbering! The output formatting should be the same as the input: ", context.map(|v|format!("{}. ", v)).unwrap_or_default(), target);
    for (i, q) in queries.iter().enumerate() {
        query.push_str(&format!("\n{}: {}", i + 1, q));
    }
    Ok(query)
}

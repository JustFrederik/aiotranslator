pub enum Context {
    /// Aditional info for chatgpt for enhance the translation.
    /// Example: The following text is a conversation between two people.
    ChatGPT(String),
}

/// Extracts the context for chatgpt from an array of Contexts
pub fn get_gpt_context(context_data: &[Context]) -> Option<&str> {
    let mut context = None;
    for c in context_data {
        context = match c {
            Context::ChatGPT(v) => Some(v.as_ref()),
            _ => None,
        };
        if context.is_some() {
            break;
        }
    }
    context
}

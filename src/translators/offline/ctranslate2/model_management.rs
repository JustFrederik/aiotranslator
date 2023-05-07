use crate::error::Error;
use crate::translators::offline::ctranslate2::tokenizer::Tokenizer;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(PartialEq, Eq)]
pub enum ModelLifetime {
    Dispose,
    KeepAlive,
}
pub struct Models {
    pub mode: ModelLifetime,
    pub tokenizers: HashMap<String, Tokenizer>,
}

impl Default for Models {
    fn default() -> Self {
        Self::new(ModelLifetime::KeepAlive)
    }
}

impl Models {
    pub fn new(model_lifetime: ModelLifetime) -> Self {
        Self {
            mode: model_lifetime,
            tokenizers: HashMap::new(),
        }
    }

    pub fn get_tokenizer(&mut self, ident: &str, path: PathBuf) -> Result<&Tokenizer, Error> {
        let v = self.tokenizers.get(ident);
        if v.is_none() {
            let tokenizer = Tokenizer::new(&path, ident.to_string());
            self.tokenizers.insert(ident.to_string(), tokenizer);
            return self
                .tokenizers
                .get(ident)
                .ok_or_else(|| Error::new_option("Tokenizer not found"));
        }
        //FIXME: duplicate search due to borrow checker
        self.tokenizers
            .get(ident)
            .ok_or_else(|| Error::new_option("Tokenizer not found"))
    }

    pub fn cleanup(&mut self) {
        //TODO: remove only model that was created(multithreaded)
        if self.mode == ModelLifetime::Dispose {
            self.tokenizers.clear();
        }
    }
}

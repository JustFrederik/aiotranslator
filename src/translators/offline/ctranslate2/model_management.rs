use crate::error::Error;
use crate::translators::offline::ctranslate2::tokenizer::Tokenizer;
use crate::translators::offline::ctranslate2::Device;
use rustyctranslate2::CTranslator;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(PartialEq, Eq)]
pub enum ModelLifetime {
    Dispose,
    KeepAlive,
}
pub struct TokenizerModels {
    pub mode: ModelLifetime,
    pub tokenizers: HashMap<String, Tokenizer>,
}

pub struct CTranslateModels {
    pub mode: ModelLifetime,
    pub ctranslate2_models: HashMap<String, CTranslator>,
}

impl CTranslateModels {
    pub fn new(model_lifetime: ModelLifetime) -> Self {
        Self {
            mode: model_lifetime,
            ctranslate2_models: HashMap::new(),
        }
    }
    pub fn get_translator(
        &mut self,
        ident: &str,
        path: PathBuf,
        device: &Device,
        compressed: bool,
    ) -> Result<&mut CTranslator, Error> {
        let v = self.ctranslate2_models.get(ident);
        if v.is_none() {
            let ctranslate2_model =
                CTranslator::new(path, device.is_cuda(), compressed).map_err(Error::new_option)?;
            self.ctranslate2_models
                .insert(ident.to_string(), ctranslate2_model);
            return self
                .ctranslate2_models
                .get_mut(ident)
                .ok_or_else(|| Error::new_option("CTranslate2 model not found"));
        }
        self.ctranslate2_models
            .get_mut(ident)
            .ok_or_else(|| Error::new_option("CTranslate2 not found"))
    }
    pub fn cleanup(&mut self) {
        //TODO: remove only model that was created(multithreaded)
        if self.mode == ModelLifetime::Dispose {
            self.ctranslate2_models.clear();
        }
    }
}

impl Default for TokenizerModels {
    fn default() -> Self {
        Self::new(ModelLifetime::KeepAlive)
    }
}

impl Default for CTranslateModels {
    fn default() -> Self {
        Self::new(ModelLifetime::KeepAlive)
    }
}

impl TokenizerModels {
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
        //FIXME: duplicate get due to borrow checker
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

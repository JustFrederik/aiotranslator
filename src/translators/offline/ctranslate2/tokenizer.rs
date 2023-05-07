use std::path::PathBuf;

use sentencepiece::SentencePieceProcessor;

use crate::error::Error;

pub struct Tokenizer {
    spp: SentencePieceProcessor,
    pub ident: String,
}

impl Tokenizer {
    pub fn new(path: &PathBuf, ident: String) -> Self {
        let spp = SentencePieceProcessor::open(path).unwrap();
        Self { spp, ident }
    }

    pub fn tokenize(&self, text: &[String]) -> Result<Vec<Vec<String>>, Error> {
        text.iter()
            .map(|v| {
                self.spp
                    .encode(v)
                    .map(|v| v.iter().map(|v| v.piece.to_string()).collect::<Vec<_>>())
                    .map_err(|e| Error::new("Sentencepiecerror", e))
            })
            .collect()
    }

    pub fn detokenize(&self, tokens: Vec<Vec<String>>) -> Result<Vec<String>, Error> {
        tokens
            .into_iter()
            .map(|v| {
                self.spp
                    .decode_pieces(&v)
                    .map_err(|e| Error::new("Sentencepiecerror", e))
            })
            .collect()
    }
}

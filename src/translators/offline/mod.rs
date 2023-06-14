#[cfg(feature = "ctranslate_req")]
pub mod ctranslate2;
#[cfg(feature = "jparacrawl")]
pub mod jparacrawl;
#[cfg(feature = "m2m100")]
pub mod m2m100;
#[cfg(feature = "nllb")]
pub mod nllb;
#[cfg(feature = "sugoi")]
pub mod sugoi;

#[derive(PartialEq, Eq, Copy, Clone, Default, Debug)]
#[cfg(feature = "ctranslate_req")]
pub enum ModelFormat {
    #[default]
    Compact,
    Normal,
}
#[cfg(feature = "ctranslate_req")]
impl ModelFormat {
    fn is_compressed(&self) -> bool {
        if self == &ModelFormat::Compact {
            return true;
        }
        false
    }
}

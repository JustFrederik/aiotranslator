#[cfg(feature = "offline_req")]
pub mod ctranslate2;
#[cfg(feature = "jparacrawl")]
pub mod jparacrawl;
#[cfg(feature = "m2m100")]
pub mod m2m100;
#[cfg(feature = "sugoi")]
pub mod sugoi;

#[derive(PartialEq, Eq)]
#[cfg(feature = "offline_req")]
pub enum ModelFormat {
    Compact,
    Normal,
}
#[cfg(feature = "offline_req")]
impl ModelFormat {
    fn is_compressed(&self) -> bool {
        if self == &ModelFormat::Compact {
            return true;
        }
        false
    }
}

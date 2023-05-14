mod baidu;
#[cfg(feature = "chatgpt")]
pub mod chatgpt;
#[cfg(feature = "deepl")]
pub mod deepl;
pub mod google;
#[cfg(feature = "libre")]
pub mod libretranslate;
#[cfg(feature = "mymemory")]
pub mod mymemory;
mod papago;
mod youdao;

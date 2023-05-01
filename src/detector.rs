#[cfg(feature = "lingua-detector")]
use lingua::{Language as LinguaLanguage, LanguageDetector, LanguageDetectorBuilder};
#[cfg(feature = "whatlang-detector")]
use whatlang::detect;
#[cfg(feature = "whatlang-detector")]
use whatlang::Lang;

use crate::error::Error;
use crate::languages::Language;

/// list of supported detectors
pub enum Detectors {
    #[cfg(feature = "lingua-detector")]
    /// Very slow, using https://github.com/pemistahl/lingua-rs
    Lingua,
    #[cfg(feature = "whatlang-detector")]
    /// Fast using https://github.com/greyblake/whatlang-rs
    Whatlang,
}

/// chooses the detector
pub fn detect_language(text: &str, detectors: &Detectors) -> Result<Language, Error> {
    //TODO: implement api detectors
    match detectors {
        #[cfg(feature = "lingua-detector")]
        Detectors::Lingua => detect_language_lingua(text),
        #[cfg(feature = "whatlang-detector")]
        Detectors::Whatlang => detect_language_whatlang(text),
        _ => Err(Error::new_option("No detector available")),
    }
}

/// returns the LanguageCode enum of the text using lingua
#[cfg(feature = "lingua-detector")]
pub fn detect_language_lingua(text: &str) -> Result<Language, Error> {
    let detector: LanguageDetector = LanguageDetectorBuilder::from_all_languages().build();
    let detected_language: Option<LinguaLanguage> = detector.detect_language_of(text);

    match &detected_language {
        Some(lang) => Ok(match lang {
            LinguaLanguage::Afrikaans => Language::Afrikaans,
            LinguaLanguage::Albanian => Language::Albanian,
            LinguaLanguage::Arabic => Language::Arabic,
            LinguaLanguage::Armenian => Language::Armenian,
            LinguaLanguage::Azerbaijani => Language::Azerbaijani,
            LinguaLanguage::Basque => Language::Basque,
            LinguaLanguage::Belarusian => Language::Belarusian,
            LinguaLanguage::Bengali => Language::Bengali,
            LinguaLanguage::Bokmal => Language::Bokmål,
            LinguaLanguage::Bosnian => Language::Bosnian,
            LinguaLanguage::Bulgarian => Language::Bulgarian,
            LinguaLanguage::Catalan => Language::Catalan,
            LinguaLanguage::Chinese => Language::Chinese,
            LinguaLanguage::Croatian => Language::Croatian,
            LinguaLanguage::Czech => Language::Czech,
            LinguaLanguage::Danish => Language::Danish,
            LinguaLanguage::Dutch => Language::Dutch,
            LinguaLanguage::English => Language::English,
            LinguaLanguage::Esperanto => Language::Spanish,
            LinguaLanguage::Estonian => Language::Estonian,
            LinguaLanguage::Finnish => Language::Finnish,
            LinguaLanguage::French => Language::French,
            LinguaLanguage::Ganda => Language::Ganda,
            LinguaLanguage::Georgian => Language::Georgian,
            LinguaLanguage::German => Language::German,
            LinguaLanguage::Greek => Language::Greek,
            LinguaLanguage::Gujarati => Language::Gujarati,
            LinguaLanguage::Hebrew => Language::Hebrew,
            LinguaLanguage::Hindi => Language::Hindi,
            LinguaLanguage::Hungarian => Language::Hungarian,
            LinguaLanguage::Icelandic => Language::Icelandic,
            LinguaLanguage::Indonesian => Language::Indonesian,
            LinguaLanguage::Irish => Language::Irish,
            LinguaLanguage::Italian => Language::Italian,
            LinguaLanguage::Japanese => Language::Japanese,
            LinguaLanguage::Kazakh => Language::Kazakh,
            LinguaLanguage::Korean => Language::Korean,
            LinguaLanguage::Latin => Language::Latin,
            LinguaLanguage::Latvian => Language::Latvian,
            LinguaLanguage::Lithuanian => Language::Lithuanian,
            LinguaLanguage::Macedonian => Language::Macedonian,
            LinguaLanguage::Malay => Language::Malayalam,
            LinguaLanguage::Maori => Language::Maori,
            LinguaLanguage::Marathi => Language::Marathi,
            LinguaLanguage::Mongolian => Language::Mongolian,
            LinguaLanguage::Nynorsk => Language::Nynorsk,
            LinguaLanguage::Persian => Language::Persian,
            LinguaLanguage::Polish => Language::Polish,
            LinguaLanguage::Portuguese => Language::Portuguese,
            LinguaLanguage::Punjabi => Language::Panjabi,
            LinguaLanguage::Romanian => Language::Romanian,
            LinguaLanguage::Russian => Language::Russian,
            LinguaLanguage::Serbian => Language::Serbian,
            LinguaLanguage::Shona => Language::Shona,
            LinguaLanguage::Slovak => Language::Slovak,
            LinguaLanguage::Slovene => Language::Slovenian,
            LinguaLanguage::Somali => Language::Somali,
            LinguaLanguage::Sotho => Language::Sotho,
            LinguaLanguage::Spanish => Language::Spanish,
            LinguaLanguage::Swahili => Language::Swahili,
            LinguaLanguage::Swedish => Language::Swedish,
            LinguaLanguage::Tagalog => Language::Tagalog,
            LinguaLanguage::Tamil => Language::Tamil,
            LinguaLanguage::Telugu => Language::Telugu,
            LinguaLanguage::Thai => Language::Thai,
            LinguaLanguage::Tsonga => Language::Tsonga,
            LinguaLanguage::Tswana => Language::Tswana,
            LinguaLanguage::Turkish => Language::Turkish,
            LinguaLanguage::Ukrainian => Language::Ukrainian,
            LinguaLanguage::Urdu => Language::Urdu,
            LinguaLanguage::Vietnamese => Language::Vietnamese,
            LinguaLanguage::Welsh => Language::Welsh,
            LinguaLanguage::Xhosa => Language::Xhosa,
            LinguaLanguage::Yoruba => Language::Yoruba,
            LinguaLanguage::Zulu => Language::Zulu,
        }),
        None => Err(Error::new_option("Coudlnt detect language with lingua")),
    }
}

/// returns LanguageCode enum generated from Whatlang
#[cfg(feature = "whatlang-detector")]
pub fn detect_language_whatlang(text: &str) -> Result<Language, Error> {
    let info = match detect(text) {
        Some(info) => Ok(info),
        None => Err(Error::new_option("Unknown iso")),
    }?;
    Ok(match info.lang() {
        Lang::Epo => Language::Spanish,
        Lang::Eng => Language::English,
        Lang::Rus => Language::Russian,
        Lang::Cmn => Language::Chinese,
        Lang::Spa => Language::Spanish,
        Lang::Por => Language::Portuguese,
        Lang::Ita => Language::Italian,
        Lang::Ben => Language::Bengali,
        Lang::Fra => Language::French,
        Lang::Deu => Language::German,
        Lang::Ukr => Language::Ukrainian,
        Lang::Kat => Language::Georgian,
        Lang::Ara => Language::Arabic,
        Lang::Hin => Language::Hindi,
        Lang::Jpn => Language::Japanese,
        Lang::Heb => Language::Hebrew,
        Lang::Yid => Language::Yiddish,
        Lang::Pol => Language::Polish,
        Lang::Amh => Language::Amharic,
        Lang::Jav => Language::Javanese,
        Lang::Kor => Language::Korean,
        Lang::Nob => Language::Bokmål,
        Lang::Dan => Language::Danish,
        Lang::Swe => Language::Swedish,
        Lang::Fin => Language::Finnish,
        Lang::Tur => Language::Turkish,
        Lang::Nld => Language::Dutch,
        Lang::Hun => Language::Hungarian,
        Lang::Ces => Language::Czech,
        Lang::Ell => Language::Greek,
        Lang::Bul => Language::Bulgarian,
        Lang::Bel => Language::Belarusian,
        Lang::Mar => Language::Marathi,
        Lang::Kan => Language::Kannada,
        Lang::Ron => Language::Romanian,
        Lang::Slv => Language::Slovenian,
        Lang::Hrv => Language::Croatian,
        Lang::Srp => Language::Serbian,
        Lang::Mkd => Language::Macedonian,
        Lang::Lit => Language::Lithuanian,
        Lang::Lav => Language::Latvian,
        Lang::Est => Language::Estonian,
        Lang::Tam => Language::Tamil,
        Lang::Vie => Language::Vietnamese,
        Lang::Urd => Language::Urdu,
        Lang::Tha => Language::Thai,
        Lang::Guj => Language::Gujarati,
        Lang::Uzb => Language::Uzbek,
        Lang::Pan => Language::Panjabi,
        Lang::Aze => Language::Azerbaijani,
        Lang::Ind => Language::Indonesian,
        Lang::Tel => Language::Telugu,
        Lang::Pes => Language::Persian,
        Lang::Mal => Language::Malagasy,
        Lang::Ori => Language::Oriya,
        Lang::Mya => Language::Burmese,
        Lang::Nep => Language::Nepali,
        Lang::Sin => Language::Sinhala,
        Lang::Khm => Language::CentralKhmer,
        Lang::Tuk => Language::Turkmen,
        Lang::Aka => Language::Akan,
        Lang::Zul => Language::Zulu,
        Lang::Sna => Language::Shona,
        Lang::Afr => Language::Afrikaans,
        Lang::Lat => Language::Latin,
        Lang::Slk => Language::Slovenian,
        Lang::Cat => Language::Catalan,
        Lang::Tgl => Language::Tagalog,
        Lang::Hye => Language::Armenian,
    })
}

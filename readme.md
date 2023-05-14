# Rust Text Translation Library

This Rust library provides a simple and easy-to-use interface for translating text. It can be used to chain
translations, translate with different translators, and use a special translator based on the detected language. The
library currently supports translation methods that involve making API requests to sites like Bing Translate, Google
Translate, and other online translators.

## Features

- whatlang-detector
- lingua-detector
- all-detectors
  <br/><br/>
- retries
  <br/><br/>
- deepl
- mymemory
- chatgpt
- libre
- api
  <br/><br/>
- papago-scrape
- google-scrape
- youdao-scrape
- edge-gpt-scrape
- baidu-scrape
- bing-scrape
- scraper
  <br/><br/>
- online

## Usage

Heres an example how to translate text from unkown language to english. When the detec.ted language is chinese it uses
the papago translator.
When the language isnt defined it uses the default translator which is google in this case

```rust
fn main() {
    dotenv().ok();
    let mut hashmap = HashMap::new();
    hashmap.insert(Language::Chinese, Translator::Papago);
    let selector = TranslatorSelectorInfo::Selective(
        hashmap,
        TranslatorInfo {
            translator: Translator::Google,
            to: Language::English,
        },
    );
    let v = Translators::new(
        Some(Tokens::get_env().unwrap()),
        selector,
        None,
        Some(3),
        Detectors::Whatlang,
    )
        .await
        .unwrap();
    let chatgpt_context = Context::ChatGPT("This is a text about ...".to_string());
    let translation = v.translate("Hello world".to_string(), None, &[chatgpt_context]).await.unwrap();
    let translations = v.translate_vec(vec!["Hello world".to_string(), "This is a test".to_string()], None, &[]).await.unwrap();
    println!("{:?}, {:?}", translation, translations);
}
```

The detector could be used seperatly like this:
Online detectors will be implemented oin the future

```rust
let text = "Hallo Welt";
let lang = detector::detect_language(text, & Detectors::Whatlang).unwrap();
println!("{:?}", lang);
```

## Supported Translators

🔴 = Offline, 🌐️ = Online, ✔️ = Supported, ⏱️ = not implemented yet, ❌ = does not exist, ❓ = WIP

| Translator                                                                                    | Kind | Scraped | API | Note                     |
|-----------------------------------------------------------------------------------------------|------|---------|-----|--------------------------|
| [Baidu Translate](https://fanyi.baidu.com)                                                    | 🌐️  | ✔️      | ❓️  |                          |
| [Bing Translate](https://www.bing.com/translator/)                                            | 🌐️  | ✔️      | ⏱️  |                          |
| [ChatGPT](https://openai.com/blog/chatgpt)                                                    | 🌐️  | ✔️      | ✔️  |                          |
| [Google Translate](https://translate.google.com)                                              | 🌐️  | ✔️      | ❓   |                          |
| [Papago](https://papago.naver.com)                                                            | 🌐️  | ✔️      | ❓   |                          |
| [Youdao](https://fanyi.youdao.com/index.html)                                                 | 🌐️  | ✔️      | ❓   |                          |
| [Libretranslate](https://libretranslate.com)                                                  | 🌐️  | ❌       | ✔️  |                          |
| [Mymemory](https://mymemory.translated.net)                                                   | 🌐️  | ❌       | ✔️  |                          |
| [Deepl](https://www.deepl.com/translator)                                                     | 🌐️  | ⏱️      | ✔️  |                          |
| [M2M100](https://github.com/facebookresearch/fairseq/tree/main/examples/m2m_100)              | 🔴️  | ❌️      | ️ ❌ | Converted 05/13/23       |
| [JParaCrawl](https://www.kecl.ntt.co.jp/icl/lirg/jparacrawl/)                                 | 🔴️  | ️  ❌    | ️❌  | V3                       |
| [Sugoi](https://www.patreon.com/mingshiba) <sup>[[online]](https://sugoitranslator.com)</sub> | 🔴️  | ⏱️️     | ❌️  | V4 / Support the creator |
| [Nllb](https://huggingface.co/facebook/nllb-200-distilled-600M)                               | 🔴️  | ️  ❌    | ️❌  | Converted 05/13/23       |

## Supported Languages

The supported languages can be found in `languages.csv`. This file is used to generate languages.rs. `missing` contains
the languages that are missing in `languages.csv`(languages i didn't find a name for)

## Contributing

If you find a bug or have an idea for a new feature, feel free to open an issue or submit a pull request. We welcome
contributions from the community!

use serde::Deserialize;

/// Tokens for the translators
#[derive(Deserialize, Debug)]
pub struct Tokens {
    /// GPT token
    pub gpt_token: Option<String>,
    /// GPT4 alternative API url
    pub gpt_proxy: Option<String>,
    /// GPT3 alternative API url
    pub gpt_old_proxy: Option<String>,
    /// Deepl token
    pub deepl_token: Option<String>,
    /// Libre token
    pub libre_token: Option<String>,
    /// id or key for youdao
    pub youdao_key: Option<String>,
    /// youdao secret
    pub youdao_secret: Option<String>,
    /// baidu appid for baidu translator
    pub baidu_appid: Option<String>,
    /// baidu key for baidu translator
    pub baidu_key: Option<String>,
}

impl Tokens {
    /// Gets the tokens from the environment, recommended to use dotenv
    pub fn get_env() -> Result<Self, envy::Error> {
        envy::from_env::<Tokens>()
    }
}

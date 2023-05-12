use std::collections::HashSet;
use std::str::FromStr;

use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::error::Error;
use crate::languages::Language;
use crate::translators::helpers::option_error;
use crate::translators::tokens::Tokens;
use crate::translators::translator_structure::{
    TranslationOutput, TranslationVecOutput, TranslatorLanguages, TranslatorNoContext,
};

pub struct GoogleTranslator {
    host: String,
    rpcids: String,
    bl: String,
    soc_app: i32,
    platform: i32,
    device: i32,
    rt: char,
}

impl Default for GoogleTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
#[cfg(feature = "fetch_languages")]
impl TranslatorLanguages for GoogleTranslator {
    async fn get_languages(client: &Client, _: &Tokens) -> Result<Vec<String>, Error> {
        let se = Self::new();
        let html = client
            .get(se.host)
            .send()
            .await
            .map_err(|e| Error::new("failed request", e))?
            .text()
            .await
            .map_err(|e| Error::new("failed to convert request to text", e))?;
        let re = regex::Regex::new(r#"data-language-code="(.*?)""#)
            .map_err(|e| Error::new("failed regex", e))?;
        let res = re
            .captures_iter(&html)
            .map(|capture| capture[1].to_string())
            .collect::<Vec<_>>();
        let mut res2 = HashSet::new();
        for v in res {
            res2.insert(v);
        }

        res2.remove("auto");
        Ok(res2.into_iter().collect())
    }
}

#[async_trait]
impl TranslatorNoContext for GoogleTranslator {
    async fn translate(
        &self,
        client: &Client,
        query: &str,
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationOutput, Error> {
        let v = self
            .translate_vec(client, &[query.to_string()], from, to)
            .await?;
        Ok(TranslationOutput {
            text: v.text.join("\\n"),
            lang: v.lang,
        })
    }

    async fn translate_vec(
        &self,
        client: &Client,
        query: &[String],
        from: Option<Language>,
        to: &Language,
    ) -> Result<TranslationVecOutput, Error> {
        let vv = self.fetch(client, from, to, &query.join("\\n")).await?;
        let language = vv
            .last()
            .ok_or_else(|| Error::new_option("No language found"))?
            .to_string();
        let temp: Vec<Value> = serde_json::from_value(vv[1][0][0][5].clone())
            .map_err(|e| Error::new("failed serde_json", e))?;
        let mut translations = vec![];
        for v in &temp {
            let temp = v[0].to_string();
            translations.push(temp[1..temp.len() - 1].to_string());
        }
        let res = translations
            .join(" ")
            .split("\\n ")
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        if language.to_lowercase() == *"null" {
            return Err(Error::new_option("Value is null"));
        }
        Ok(TranslationVecOutput {
            text: res,
            lang: Language::from_str(&language[1..language.len() - 1])?,
        })
    }
}

impl GoogleTranslator {
    //TODO: other urls
    //'translate.google.ac' 'translate.google.ad' 'translate.google.ae'
    // 'translate.google.al' 'translate.google.am' 'translate.google.as'
    // 'translate.google.at' 'translate.google.az' 'translate.google.ba'
    // 'translate.google.be' 'translate.google.bf' 'translate.google.bg'
    // 'translate.google.bi' 'translate.google.bj' 'translate.google.bs'
    // 'translate.google.bt' 'translate.google.by' 'translate.google.ca'
    // 'translate.google.cat' 'translate.google.cc' 'translate.google.cd'
    // 'translate.google.cf' 'translate.google.cg' 'translate.google.ch'
    // 'translate.google.ci' 'translate.google.cl' 'translate.google.cm'
    // 'translate.google.cn' 'translate.google.co.ao' 'translate.google.co.bw'
    // 'translate.google.co.ck' 'translate.google.co.cr' 'translate.google.co.id'
    // 'translate.google.co.il' 'translate.google.co.in' 'translate.google.co.jp'
    // 'translate.google.co.ke' 'translate.google.co.kr' 'translate.google.co.ls'
    // 'translate.google.co.ma' 'translate.google.co.mz' 'translate.google.co.nz'
    // 'translate.google.co.th' 'translate.google.co.tz' 'translate.google.co.ug'
    // 'translate.google.co.uk' 'translate.google.co.uz' 'translate.google.co.ve'
    // 'translate.google.co.vi' 'translate.google.co.za' 'translate.google.co.zm'
    // 'translate.google.co.zw' 'translate.google.com.af' 'translate.google.com.ag'
    // 'translate.google.com.ai' 'translate.google.com.ar' 'translate.google.com.au'
    // 'translate.google.com.bd' 'translate.google.com.bh' 'translate.google.com.bn'
    // 'translate.google.com.bo' 'translate.google.com.br' 'translate.google.com.bz'
    // 'translate.google.com.co' 'translate.google.com.cu' 'translate.google.com.cy'
    // 'translate.google.com.do' 'translate.google.com.ec' 'translate.google.com.eg'
    // 'translate.google.com.et' 'translate.google.com.fj' 'translate.google.com.gh'
    // 'translate.google.com.gi' 'translate.google.com.gt' 'translate.google.com.hk'
    // 'translate.google.com.jm' 'translate.google.com.kh' 'translate.google.com.kw'
    // 'translate.google.com.lb' 'translate.google.com.ly' 'translate.google.com.mm'
    // 'translate.google.com.mt' 'translate.google.com.mx' 'translate.google.com.my'
    // 'translate.google.com.na' 'translate.google.com.ng' 'translate.google.com.ni'
    // 'translate.google.com.np' 'translate.google.com.om' 'translate.google.com.pa'
    // 'translate.google.com.pe' 'translate.google.com.pg' 'translate.google.com.ph'
    // 'translate.google.com.pk' 'translate.google.com.pr' 'translate.google.com.py'
    // 'translate.google.com.qa' 'translate.google.com.sa' 'translate.google.com.sb'
    // 'translate.google.com.sg' 'translate.google.com.sl' 'translate.google.com.sv'
    // 'translate.google.com.tj' 'translate.google.com.tr' 'translate.google.com.tw'
    // 'translate.google.com.ua' 'translate.google.com.uy' 'translate.google.com.vc'
    // 'translate.google.com.vn' 'translate.google.com' 'translate.google.cv'
    // 'translate.google.cz' 'translate.google.de' 'translate.google.dj'
    // 'translate.google.dk' 'translate.google.dm' 'translate.google.dz'
    // 'translate.google.ee' 'translate.google.es' 'translate.google.eu'
    // 'translate.google.fi' 'translate.google.fm' 'translate.google.fr'
    // 'translate.google.ga' 'translate.google.ge' 'translate.google.gf'
    // 'translate.google.gg' 'translate.google.gl' 'translate.google.gm'
    // 'translate.google.gp' 'translate.google.gr' 'translate.google.gy'
    // 'translate.google.hn' 'translate.google.hr' 'translate.google.ht'
    // 'translate.google.hu' 'translate.google.ie' 'translate.google.im'
    // 'translate.google.io' 'translate.google.iq' 'translate.google.is'
    // 'translate.google.it' 'translate.google.je' 'translate.google.jo'
    // 'translate.google.kg' 'translate.google.ki' 'translate.google.kz'
    // 'translate.google.la' 'translate.google.li' 'translate.google.lk'
    // 'translate.google.lt' 'translate.google.lu' 'translate.google.lv'
    // 'translate.google.md' 'translate.google.me' 'translate.google.mg'
    // 'translate.google.mk' 'translate.google.ml' 'translate.google.mn'
    // 'translate.google.ms' 'translate.google.mu' 'translate.google.mv'
    // 'translate.google.mw' 'translate.google.ne' 'translate.google.nf'
    // 'translate.google.nl' 'translate.google.no' 'translate.google.nr'
    // 'translate.google.nu' 'translate.google.pl' 'translate.google.pn'
    // 'translate.google.ps' 'translate.google.pt' 'translate.google.ro'
    // 'translate.google.rs' 'translate.google.ru' 'translate.google.rw'
    // 'translate.google.sc' 'translate.google.se' 'translate.google.sh'
    // 'translate.google.si' 'translate.google.sk' 'translate.google.sm'
    // 'translate.google.sn' 'translate.google.so' 'translate.google.sr'
    // 'translate.google.st' 'translate.google.td' 'translate.google.tg'
    // 'translate.google.tk' 'translate.google.tl' 'translate.google.tm'
    // 'translate.google.tn' 'translate.google.to' 'translate.google.tt'
    // 'translate.google.us' 'translate.google.vg' 'translate.google.vu'
    // 'translate.google.ws'
    pub fn new() -> Self {
        Self {
            host: "https://translate.google.com".to_string(),
            rpcids: "MkEWBc".to_string(),
            bl: "boq_translate-webserver_20201207.13_p0".to_string(),
            soc_app: 1,
            platform: 1,
            device: 1,
            rt: 'c',
        }
    }

    pub async fn fetch(
        &self,
        client: &Client,
        from: Option<Language>,
        to: &Language,
        text: &str,
    ) -> Result<Vec<Value>, Error> {
        let data = self.generate_data(
            text,
            &option_error(from.map(|v| v.to_google_str()))?.unwrap_or_else(|| "auto".to_string()),
            &to.to_google_str()?,
        )?;
        let v = client.post(format!("{}/_/TranslateWebserverUi/data/batchexecute?rpcids={}&bl={}&soc-app={}&soc-platform={}&soc-device={}&rt={}", self.host, self.rpcids, self.bl, self.soc_app, self.platform, self.device, self.rt))
            .header("Content-Type", "application/x-www-form-urlencoded;charset=UTF-8")
            .body(data)
            .send().await;
        let text = v
            .map_err(|e| Error::new("Request failed", e))?
            .text()
            .await
            .map_err(|e| Error::new("Parsing response failed", e))?;
        let v: Vec<Vec<Value>> = serde_json::from_str(
            text.split('\n')
                .nth(3)
                .ok_or_else(|| Error::new_option("Output format errro"))?,
        )
        .map_err(|e| Error::new("Failed to serialize", e))?;
        let vv: Vec<Value> = serde_json::from_str(
            v.get(0)
                .ok_or_else(|| Error::new_option("Output format errror"))?
                .get(2)
                .ok_or_else(|| Error::new_option("Output format errro"))?
                .as_str()
                .ok_or_else(|| Error::new_option("Output format errro"))?,
        )
        .map_err(|e| Error::new("Failed to serialize serde_json", e))?;
        Ok(vv)
    }

    //TODO: implement feature
    pub async fn get_pronouciation(
        &self,
        client: &Client,
        from: Option<Language>,
        text: &str,
    ) -> Result<(), Error> {
        let vv = self.fetch(client, from, &Language::English, text).await?;
        let language = vv
            .last()
            .ok_or_else(|| Error::new_option("No language found"))?
            .to_string();
        let pronouciation = vv[0][0].to_string();
        if language.to_lowercase() == *"null" || pronouciation.to_lowercase() == *"null" {
            return Err(Error::new_option("Value is null"));
        }
        println!("{} {}", language, pronouciation);
        Ok(())
    }

    pub fn generate_data(&self, text: &str, from: &str, to: &str) -> Result<String, Error> {
        let v1 = "MkEWBc".to_string();
        let v2 = format!(
            "[[\\\"{}\\\",\\\"{}\\\",\\\"{}\\\",true],[null]]",
            text.replace('\n', "\\n").replace('\\', "\\\\"),
            from,
            to
        );
        let v3 = "null".to_string();
        let v4 = "generic".to_string();
        let query = [(
            "f.req",
            format!("[[[\"{}\",\"{}\",{},\"{}\"]]]", v1, v2, v3, v4),
        )];
        serde_urlencoded::to_string(query).map_err(|v| Error::new("Failed to serialize", v))
    }
}

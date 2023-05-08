use std::collections::HashMap;
use std::io::Write;

use codegen::{Function, Variant};

/// List of all languages
#[derive(Debug, serde::Deserialize)]
pub struct Records {
    pub headers: Vec<String>,
    pub records: Vec<Vec<Option<String>>>,
}

impl Records {
    /// Deserialize csv file
    pub fn new() -> Result<Self, String> {
        let text = include_str!("../languages.csv");
        let mut rdr = csv::Reader::from_reader(text.as_bytes());
        let headers = rdr
            .headers()
            .unwrap()
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>();

        let mut records: Vec<Vec<Option<String>>> = Vec::new();
        for result in rdr.deserialize() {
            records.push(result.map_err(|v| v.to_string())?);
        }
        Ok(Records { headers, records })
    }

    pub fn add_line(&mut self, header: &str, values: &[String]) {
        self.headers.push(header.to_string());
        for row in self.records.iter_mut() {
            // let mut pushthis = None;
            // for v in values {
            //     let v_short = v.split('_').collect::<Vec<_>>().first().unwrap().to_string();
            //     if row.contains(&Some(v_short)){
            //         pushthis = Some(v.to_string());
            //         break;
            //     }
            // }
            // row.push(pushthis);
            let v = values
                .iter()
                .find(|value| row.contains(&Some(value.to_string())))
                .map(|v| v.to_string());
            row.push(v);
        }
    }

    pub fn write_file(&mut self, filename: &str) -> Result<(), String> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.serialize(&self.headers).unwrap();
        self.records
            .sort_by_key(|s| s.first().as_ref().unwrap().as_ref().unwrap().to_string());
        for t in &self.records {
            wtr.serialize(t).unwrap();
        }
        wtr.flush().unwrap();
        let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        std::fs::write(filename, data).unwrap();
        Ok(())
    }

    fn format_data(&self) -> HashMap<String, Vec<(String, String)>> {
        let mut res = HashMap::new();
        for category in self.headers.iter().enumerate() {
            let v = self
                .records
                .iter()
                .filter_map(|v| {
                    v[category.0]
                        .as_ref()
                        .map(|vv| (Self::format_to_enum(v[0].as_ref().unwrap()), vv.to_string()))
                })
                .collect::<Vec<_>>();
            res.insert(category.1.to_string(), v);
        }
        res
    }

    pub fn generate_file(&self) -> Result<(), std::io::Error> {
        use codegen::Scope;
        let mut scope = Scope::new();

        let v = scope.new_enum("Language");

        let mut func = Function::new("from_str");
        func.arg("s", "&str");
        func.ret("Result<Self, Self::Err>");
        func.line("match s {");

        for record in &self.records {
            println!("{:?}", record[0].as_ref());
            let enum_name = Self::format_to_enum(record[0].as_ref().unwrap());
            let mut var = Variant::new(&enum_name);
            var.annotation(&format!("/// Code: {}", record[1].as_ref().unwrap()));
            v.push_variant(var);
            let mut seen = std::collections::HashSet::new();
            let mut line = record
                .iter()
                .filter_map(|v| v.as_ref().map(|w| format!("\"{}\"", w)))
                .collect::<Vec<String>>();
            line.retain(|x| seen.insert(x.to_string()));

            func.line(format!(
                "{} => Ok(Language::{}),",
                line.join(" | "),
                enum_name
            ));
        }
        v.push_variant(Variant::new("Unknown"));

        let im_from = scope.new_impl("FromStr for Language");

        func.line("_ => Err(Error::new_option(\"No result found\"))\n}");

        im_from.associate_type("Err", "Error");
        im_from.push_fn(func);
        let imp = scope.new_impl("Language");

        let data = self.format_data();
        for item in data.iter() {
            let get = generate_to_string_function(&format!("to_{}_str", item.0), item.1);
            imp.push_fn(get);
            if item.0 != "6391" && item.0 != "6393" && item.0 != "name" {
                let supported = generate_supported(&format!("get_supported_{}", item.0), item.1);
                imp.push_fn(supported);
            }
        }

        let mut file = std::fs::File::create("src/languages.rs")?;
        file.write_all(b"///This file is auto generated.\n\n")?;
        file.write_all(b"use crate::error::Error;\n")?;
        file.write_all(b"use std::str::FromStr;\n")?;
        file.write_all(b"#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]\n")?;
        file.write_all(b"pub ")?;
        file.write_all(scope.to_string().as_bytes())?;
        Ok(())
    }

    fn make_ascii_titlecase(s: impl ToString) -> String {
        let mut s = s.to_string();
        if let Some(r) = s.get_mut(0..1) {
            r.make_ascii_uppercase();
        }
        s
    }
    fn format_to_enum(s: &str) -> String {
        let items = match s.contains(' ') {
            true => s.split(' ').collect(),
            false => vec![s],
        };
        items
            .iter()
            .map(Self::make_ascii_titlecase)
            .collect::<Vec<_>>()
            .join("")
    }
}

fn generate_to_string_function(name: &str, f: &[(String, String)]) -> Function {
    let mut v = Function::new(name);
    v.arg_self();
    v.vis("pub");
    v.ret("Result<String, Error>");
    v.line("match self {");
    for value in f {
        v.line(format!(
            "Self::{} => Ok(\"{}\".to_string()),",
            value.0, value.1
        ));
    }
    v.line("_ =>  Err(Error::new_option(\"Translator doenst support this language\")),\n}");
    v
}

fn generate_supported(name: &str, f: &[(String, String)]) -> Function {
    let mut v = Function::new(name);
    v.vis("pub");
    v.ret("Vec<Self>");
    let items = f
        .iter()
        .map(|v| format!("Self::{}", v.0))
        .collect::<Vec<_>>()
        .join(", ");
    v.line(format!("vec![{}]", items));
    v
}

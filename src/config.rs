
use std::{collections::HashMap, fmt::Display, marker::PhantomData};

use anyhow::ensure;
use serde::{Deserialize, Serialize};

pub struct NotValidated;
pub struct Validated;
pub trait Validation {}
impl Validation for Validated {}


#[derive(Deserialize, Serialize)]
pub struct ZmConfig<S = NotValidated> {
    #[serde(default)]
    keywords: Vec<ZmKeywordConfig>,

    #[serde(skip)]
    _marker: PhantomData<fn() -> S>,
}
impl<S> Display for ZmConfig<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.keywords.is_empty() {
            writeln!(f, "keywords:\n")?;
            for k in &self.keywords {
                writeln!(f, "{}", k)?;
            }
        }
        Ok(())
    }
}

impl<S> ZmConfig<S> {
    pub fn validate(self) -> anyhow::Result<ZmConfig<Validated>> {
        let mut names: HashMap<&String, &ZmKeywordConfig> = HashMap::new();
        let mut prefixes: HashMap<&String, &ZmKeywordConfig> = HashMap::new();
        for k in &self.keywords {
            if let Some(ref prefix) = k.prefix {
                ensure!(!prefixes.contains_key(prefix),
                    "Prefix Error: prefix `{}` of \"{}\" conflicts with one of \"{}\"", prefix, k.name, prefixes[prefix].name);
                prefixes.insert(prefix, k);
            }
            if let Some(ref m) = k.mapping {
                for mk in m.keys() {
                    ensure!(!names.contains_key(mk),
                        "Keyword Error: keyword `{}` in mapping of \"{}\" conflicts with keyword \"{}\"", mk, k.name, names[mk].name);
                    names.insert(mk, k);
                }
            } else {
                ensure!(!names.contains_key(&k.name),
                    "Keyword Error: keyword `{}` conflict with \"{}\"", k.name, names[&k.name].name);
                names.insert(&k.name, k);
            }
        }
    
        Ok(ZmConfig::<Validated> {
            keywords: self.keywords,
            _marker: PhantomData
        })
    }
}

impl<S: Validation> ZmConfig<S> {
    pub fn get_keyword(&self) -> &Vec<ZmKeywordConfig> {
        &self.keywords
    }
}


#[derive(Deserialize, Serialize)]
pub struct ZmKeywordConfig {
    pub name: String,
    pub help: String,
    pub mapping: Option<HashMap<String, String>>,
    pub default: Option<String>,
    pub prefix: Option<String>,
}
impl Display for ZmKeywordConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  {}  :  {}", self.name, self.help)?;
        if matches!(self.mapping, Some(ref m) if !m.is_empty()) {
            writeln!(f, "    mapping:")?;
            for (k, v) in self.mapping.as_ref().unwrap() {
                writeln!(f, "      \"{}\"  ==>  \"{}\"", k, v)?;
            }
        }
        if let Some(ref d) = self.default {
            writeln!(f, "    default: \"{}\"", d)?;
        }
        if let Some(ref p) = self.prefix {
            writeln!(f, "    prefix: \"{}\"", p)?;
        }
        Ok(())
    }
}
impl ZmKeywordConfig {
    pub fn replace(&self, arg: &str) -> Option<String> {
        let arg = match &self.prefix {
            Some(prefix) if arg.starts_with(prefix) => {
                &arg[prefix.len()..]
            },
            _ => arg,
        };
        match &self.mapping {
            Some(m) => {
                m.get(arg).cloned()
            },
            _ if arg == self.name => {
                Some(self.name.clone())
            },
            _ => None,
        }
    }
}

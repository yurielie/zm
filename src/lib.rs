
mod config;

use std::{collections::HashSet, ops::ControlFlow, path::Path};
use anyhow::ensure;

use config::{ZmConfig, Validated};

pub struct Zm {
    config: ZmConfig<Validated>,

    join_delim: Option<String>,
}
impl Zm {

    const OPT_JOIN_DELIM: &'static str = "--join_delim";

    pub fn from_file<T>(path: T) -> anyhow::Result<Self>
    where
        T: AsRef<Path>,
    {
        let config: ZmConfig = serde_json::from_reader(std::fs::File::open(path)?)?;
        Ok(Self {
            config: config.validate()?,
            join_delim: None,
        })
    }

    pub fn set_join_delim(&mut self, delim: &str) {
        self.join_delim = Some(delim.into());
    }

    pub fn show_help(&self) {
        println!("{}", self.config);
    }

    pub fn parse(&self) -> anyhow::Result<Vec<String>> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        self.parse_args(&args)
    }

    pub fn parse_args(&self, args: &[String]) -> anyhow::Result<Vec<String>> {

        let mut join_delim = self.join_delim.clone();

        let args = if let Some(pos) = args.iter().position(|s| s == "--") {
            let zm_opts = &args[..pos];
            let mut i = 0;
            while i < zm_opts.len() {
                if zm_opts[i] == Self::OPT_JOIN_DELIM {
                    ensure!(i + 1 < zm_opts.len(), "Option Error: option `{}` requires the delimitor but not given.", Self::OPT_JOIN_DELIM);
                    join_delim = Some(zm_opts[i + 1].to_string());
                    i += 1;
                }
                i += 1;
            }

            &args[pos + 1..]
        } else {
            args
        };

        let mut keys = vec![];
        let mut res = vec![];
        let mut got = HashSet::new();

        for a in args {
            let flow = self.config.get_keyword().iter()
                .try_for_each(|k| {
                    if let Some(replaced) = k.replace(a) {
                        keys.push(k.name.clone());
                        res.push(replaced);
                        got.insert(&k.name);
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                });
            if flow.is_continue() {
                keys.push("".into());
                res.push(a.to_string());
            }
        }

        self.config.get_keyword().iter()
            .filter(|k| k.default.is_some() && !got.contains(&k.name))
            .for_each(|k| {
                keys.push(k.name.clone());
                res.push(k.default.as_ref().cloned().unwrap());
            });
        
        if let Some(ref delim) = join_delim {
            Ok(keys.into_iter().zip(res).map(|(k, r)| format!("{}{}{}", k, delim, r)).collect())
        } else {    
            Ok(res.into_iter().filter(|s| !s.is_empty()).collect())
        }
    }
}


mod config;

use std::{collections::HashSet, ops::ControlFlow, path::Path};
use anyhow::ensure;

use config::{ZmConfig, Validated};

pub struct Zm {
    config: ZmConfig<Validated>,

    join_delim: Option<String>,
}
impl Zm {

    const OPT_SHOW_KW_WITH: &'static str = "--show_keyword_with";
    const OPT_HELP_LONG: &'static str = "--help";
    const OPT_HELP_SHORT: &'static str = "-h";
    
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
        println!("Zm: v{}", env!("CARGO_PKG_VERSION"));
        println!("\nusage: zm [OPTIONS] -- [COMMANDLINE]...");
        println!("\noptions:");
        println!("  {} <DELIM>  show keyword name with given delimitor like 'keyword=value'", Self::OPT_SHOW_KW_WITH);
        println!("  {}, {}                   print help", Self::OPT_HELP_SHORT, Self::OPT_HELP_LONG);
        println!("\n{}", self.config);
    }

    pub fn parse(&self) -> anyhow::Result<Vec<String>> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        self.parse_args(&args)
    }

    pub fn parse_args(&self, args: &[String]) -> anyhow::Result<Vec<String>> {

        if args.is_empty() {
            self.show_help();
            return Ok(vec![])
        }

        let mut join_delim = self.join_delim.clone();

        let opt_pos = args.iter().position(|s| s == "--").unwrap_or(args.len());
        if opt_pos > 0 {
            let zm_opts = &args[..opt_pos];
            let mut i = 0;
            while i < zm_opts.len() {
                match zm_opts[i].as_str() {
                    Self::OPT_SHOW_KW_WITH => {
                        ensure!(i + 1 < zm_opts.len(),
                            "Option Error: option `{}` requires the delimitor but not given.", Self::OPT_SHOW_KW_WITH);
                        join_delim = Some(zm_opts[i + 1].to_string());
                        i += 1;
                    },
                    Self::OPT_HELP_LONG | Self::OPT_HELP_SHORT => {
                        self.show_help();
                        return Ok(vec![])
                    },
                    _ => {}
                }
                i += 1;
            }
        }

        let mut keys = vec![];
        let mut res = vec![];
        let mut got = HashSet::new();

        let mut i = opt_pos + 1;
        while i < args.len() {
            let a = &args[i];
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
            i += 1;
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

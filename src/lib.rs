
mod config;

use std::{collections::HashSet, ops::ControlFlow, path::Path};
use anyhow::ensure;

use config::{ZmConfig, Validated};


const OPT_CONFIG_FILE_LONG: &str = "--file";
const OPT_CONFIG_FILE_SHORT: &str = "-f";
const OPT_SHOW_KW_WITH: &str = "--show_keyword_with";
const OPT_HELP_LONG: &str = "--help";
const OPT_HELP_SHORT: &str = "-h";


fn load<P>(path: P) -> anyhow::Result<ZmConfig<Validated>>
where
    P: AsRef<Path>,
{
    let config: ZmConfig = serde_json::from_reader(std::fs::File::open(path)?)?;
    config.validate()
}

pub fn show_help() {
    println!("Zm: v{}", env!("CARGO_PKG_VERSION"));
    println!("\nusage: zm [OPTIONS] -- [COMMANDLINE]...");
    println!("\noptions:");
    println!("  {}, {} <FILE>            path of JSON configuration file", OPT_CONFIG_FILE_SHORT, OPT_CONFIG_FILE_LONG);
    println!("  {} <DELIM>  show keyword name with given delimitor like 'keyword=value'", OPT_SHOW_KW_WITH);
    println!("  {}, {}                   print help", OPT_HELP_SHORT, OPT_HELP_LONG);
}

pub fn parse() -> anyhow::Result<Vec<String>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    parse_args(&args)
}

pub fn parse_args(args: &[String]) -> anyhow::Result<Vec<String>> {

    if args.is_empty() {
        show_help();
        return Ok(vec![])
    }

    let mut join_delim = None;
    let mut config = None;
    let mut help = false;

    let mut it = args.iter().peekable();
    while it.peek().is_some_and(|&a| a != "--") {
        let opt = it.next().unwrap();
        match opt.as_str() {
            OPT_CONFIG_FILE_LONG | OPT_CONFIG_FILE_SHORT => {
                ensure!(it.peek().is_some(),
                    "Option Error: option `{}` requires a path to configuration file but not given.", opt);
                let path = it.next().unwrap();
                config = Some(load(path)?);
            },
            OPT_SHOW_KW_WITH => {
                ensure!(it.peek().is_some(),
                    "Option Error: option `{}` requires the delimitor but not given.", OPT_SHOW_KW_WITH);
                join_delim = Some(it.next().cloned().unwrap());
            },
            OPT_HELP_LONG | OPT_HELP_SHORT => {
                help = true;
            },
            _ => {}
        }
    }

    if help {
        show_help();
        if let Some(c) = config {
            println!("{}", c);
        }
        return Ok(vec![])
    }
    it.next(); // consume "--"

    let Some(config) = config else {
        return Ok(vec![])
    };

    let mut keys = vec![];
    let mut res = vec![];
    let mut got = HashSet::new();

    for a in it {
        let flow = config.get_keyword().iter()
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

    config.get_keyword().iter()
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


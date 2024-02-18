
mod config;

use std::{collections::HashSet, ops::ControlFlow, path::Path};
use anyhow::ensure;

use config::{ZmConfig, Validated};


const OPT_FILE: OptionDefinition = OptionDefinition::new("--file", "-f", "<FILE>", "path of JSON configuration file");
const OPT_SHOW_KW_WITH: OptionDefinition = OptionDefinition::new("--show_keyword_with", "", "<DELIMITOR>", "show keyword name with given delimitor like 'keyword=value'");
const OPT_HELP: OptionDefinition = OptionDefinition::new("--help", "-h", "", "print help");

const OPTIONS: [OptionDefinition; 3] = [
    OPT_FILE,
    OPT_SHOW_KW_WITH,
    OPT_HELP,
];

struct OptionDefinition {
    pub long: &'static str,
    pub short: &'static str,
    args: &'static str,
    help: &'static str,
}
impl OptionDefinition {
    const fn new(long: &'static str, short: &'static str, args: &'static str, help: &'static str) -> Self {
        Self { long, short, args, help, }
    }

    const fn header_len(&self) -> usize {
        let mut len = 0;
        len += self.long.len();
        if !self.short.is_empty() {
            len += ", ".len();
            len += self.short.len();
        }
        if !self.args.is_empty() {
            len += " ".len();
            len += self.args.len();
        }
        len += "  ".len();
        len
    }

    fn to_string_with_spaces(&self, spaces: usize) -> String {
        let mut s = String::new();
        if !self.short.is_empty() {
            s.push_str(self.short);
            s.push_str(", ");
        }
        s.push_str(self.long);
        if !self.args.is_empty() {
            s.push(' ');
            s.push_str(self.args);
        }
        format!("{}{}{}", s, " ".repeat(spaces), self.help)
    }
}


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

    if let Some(spaces) = OPTIONS.iter().map(|od| od.header_len()).max() {
        println!("\noptions:");
        for opt in OPTIONS {
            let spaces = spaces - opt.header_len() + 2;
            println!("  {}", opt.to_string_with_spaces(spaces));
        }
    }
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
            opt if opt == OPT_FILE.short || opt == OPT_FILE.long => {
                ensure!(it.peek().is_some_and(|&s| s != "--"),
                    "Option Error: option `{}` requires a path to configuration file but not given.", opt);
                config = Some(load(it.next().unwrap())?);
            },
            opt if opt == OPT_SHOW_KW_WITH.long => {
                ensure!(it.peek().is_some_and(|&s| s != "--"),
                    "Option Error: option `{}` requires the delimitor excluding \"--\" but not given.", OPT_SHOW_KW_WITH.long);
                join_delim = Some(it.next().cloned().unwrap());
            },
            opt if opt == OPT_HELP.short || opt == OPT_HELP.long => {
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
        Ok(keys.into_iter().zip(res).map(|(k, r)| {
            if !k.is_empty() {
                format!("{}{}{}", k, delim, r)
            } else {
                r
            }
        }).collect())
    } else {    
        Ok(res.into_iter().filter(|s| !s.is_empty()).collect())
    }
}


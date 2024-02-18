use std::{collections::HashMap, ops::ControlFlow};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct MiniConfig {
    name: String,
    help: String,
    default: Option<String>,
    mapping: Option<HashMap<String, String>>,
    prefix: Option<String>,
}
impl MiniConfig {
    fn parse(&self, s: &str) -> Option<String> {
        let s = match &self.prefix {
            Some(p) if s.starts_with(p) => &s[p.len()..],
            _ => s
        };
        self.mapping.as_ref().map_or_else(|| self.name.eq(s).then(|| self.name.clone()), |m| m.get(s).cloned())
    }
}
fn main() -> anyhow::Result<()> {
    let config: Vec<MiniConfig> = serde_json::from_reader(std::fs::File::open("./zm_mini.json")?)?;
    let mut delim = None;
    let mut it = std::env::args().skip(1).peekable();
    while it.peek().is_some_and(|s| s != "--") {
        let opt = it.next().unwrap();
        if opt == "-h" {
            println!("Zm: usage zm [OPTIONS] -- [COMMANDLINES]...\n");
            return Ok(())
        } else if opt == "-d" && it.peek().is_some() {
            delim = it.next();     
        }
    }
    let res: Vec<_> = it.skip(1).map(|a| {
        match config.iter().try_for_each(|k| k.parse(&a).map_or(ControlFlow::Continue(()), |mapped| ControlFlow::Break((delim.as_ref().map_or("".to_string(), |d| format!("{}{d}", &k.name)), mapped)))) {
            ControlFlow::Break((k, v)) => (k, v),
            ControlFlow::Continue(_) => ("".into(), a)
        }
    }).map(|(k, v)| format!("{k}{v}"))
        .collect();
    println!("{}", res.join(" "));
    Ok(())
}

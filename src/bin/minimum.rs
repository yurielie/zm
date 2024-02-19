use std::{collections::HashMap, ops::ControlFlow};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct MiniConfig {
    name: String,
    help: String,
    mapping: Option<HashMap<String, String>>,
    prefix: Option<String>,
}
impl std::fmt::Display for MiniConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  \"{}\"  -  {}", self.name, self.help)?;
        if let Some(ref m) = self.mapping {
            writeln!(f, "    mapping:")?;
            for (k, v) in m {
                writeln!(f, "      \"{}\"  ==>  \"{}\"", k, v)?;
            }
        }
        if let Some(ref p) = self.prefix {
            writeln!(f, "    prefix: \"{}\"", p)?;
        }
        Ok(())
    }
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
    let mut help = false;
    let mut file = None;
    let mut delim = None;
    let mut run = false;
    let mut it = std::env::args().skip(1).peekable();
    while it.peek().is_some_and(|s| s != "--") {
        let opt = it.next().unwrap();
        if opt == "-h" {
            help = true;
        } else if opt == "-d" && it.peek().is_some() {
            delim = it.next();
        } else if opt == "-f" && it.peek().is_some() {
            file = Some(it.next().unwrap());
        } else if opt == "--run" {
            run = true;
        }
    }
    let config: Vec<MiniConfig> = serde_json::from_reader(std::fs::File::open(file.unwrap_or("./zm_mini.json".into()))?)?;
    if help {
        println!("Zm: usage zm [OPTIONS] -- [COMMANDLINES]...\n");
        config.iter().for_each(|c| println!("{c}") );
        return Ok(())
    }
    let res: Vec<_> = it.skip(1).map(|a| {
        match config.iter().try_for_each(|k| k.parse(&a).map_or(ControlFlow::Continue(()), |mapped| ControlFlow::Break((delim.as_ref().map_or("".to_string(), |d| format!("{}{d}", &k.name)), mapped)))) {
            ControlFlow::Break((k, v)) => (k, v),
            ControlFlow::Continue(_) => ("".into(), a)
        }
    }).map(|(k, v)| format!("{k}{v}"))
        .collect();
    if run && !res.is_empty() {
        let mut c = std::process::Command::new(&res[0]);
        c.args(res.into_iter().skip(1))
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()?;
    } else {
        println!("{}", res.join(" "));
    }
    Ok(())
}

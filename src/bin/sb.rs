use std::{collections::{HashMap, HashSet}, fmt::Display};

use anyhow::ensure;
use clap::{CommandFactory, Parser};
use serde::Deserialize;

type Kw = HashMap<String, Vec<String>>;
type BoxedGeneratorFn = Box<dyn Fn(&Kw) -> anyhow::Result<Vec<String>>>;

#[derive(Deserialize)]
struct Config {
    name: String,
    help: String,
    #[serde(default)]
    priority: i32,
    #[serde(default)]
    requirements: Vec<String>,
}
impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  {}:    {}", self.name, self.help)?;
        if self.priority != 0 {
            writeln!(f, "    priority: {}", self.priority)?;
        }
        if !self.requirements.is_empty() {
            writeln!(f, "    requirements: [ {} ]", self.requirements.join(", "))?;
        }
        Ok(())
    }
}

#[derive(Default, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
)]
struct Mers {
    #[clap(short, long, help = "show help")]
    help: bool,
    #[clap(long, help = "show command")]
    dry: bool,
    #[clap(value_name = "PARAM_OR_COMMAND", help = "sequence of parameter like 'key=value' or command like 'build'")]
    args: Vec<String>,

    #[clap(skip)]
    handlers: HashMap<String, BoxedGeneratorFn>,
}
impl Mers {
    fn push<F>(&mut self, name: &str, f: F) -> anyhow::Result<()>
    where
        F: Fn(&Kw) -> anyhow::Result<Vec<String>> + 'static,
    {
        ensure!(!self.handlers.contains_key(name), "dup command: {name}");
        self.handlers.insert(name.into(), Box::new(f));
        Ok(())
    }

    fn gen_tasks(&self, kw: &Kw, args: &HashSet<String>, configs: &[Config]) -> anyhow::Result<Vec<String>> {
        let mut tasks = vec![];
        for c in configs.iter().filter(|c| args.contains(&c.name)) {
            ensure!(self.handlers.contains_key(&c.name), "not registerred: {}", c.name);
            ensure!(c.requirements.iter().all(|r| kw.contains_key(r)), "not req: {}", c.name);
            let cmds = self.handlers[&c.name](kw)?;
            tasks.extend(cmds);
        }
        Ok(tasks)
    }

    fn run(&self, kw: &Kw, args: &HashSet<String>, configs: &[Config]) -> anyhow::Result<()> {
        for t in self.gen_tasks(kw, args, configs)? {
            let mut it = t.split(' ');
            let res = std::process::Command::new(it.next().unwrap())
                .args(it)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output()?;
            ensure!(res.status.success(), "Error: {}", String::from_utf8_lossy(&res.stderr));
        }
        Ok(())
    }

    fn dry_run(&self, kw: &Kw, args: &HashSet<String>, configs: &[Config]) -> anyhow::Result<()> {
        Ok(self.gen_tasks(kw, args, configs)?.into_iter().for_each(|t| println!("{t}")))
    }

}

fn parse_args(argv: &[String]) -> (Kw, HashSet<String>) {
    let mut kw = Kw::new();
    let mut args = HashSet::new();
    for a in argv {
        if let Some(pos) = a.find('=') {
            kw.entry(a[..pos].to_string()).or_default().push(a[pos + 1..].to_string());
        } else {
            args.insert(a.clone());
        }
    }
    (kw, args)
}

fn main() -> anyhow::Result<()> {
    let configs: Vec<Config> = serde_json::from_reader(std::fs::File::open("./sb.json")?)?;
    let mut mers = Mers::parse();
    if mers.help {
        Mers::command().print_help()?;
        if !configs.is_empty() {
            println!("\ncommands:\n");
            configs.iter().for_each(|c| println!("{c}"));
        }
        return Ok(())
    }
    let (kw, args) = parse_args(&mers.args);
    let dry = mers.dry;
    mers.push("exec", |kw| Ok(vec![format!("{}", kw["cmd"].join(" "))]))?;

    dry.then(|| mers.dry_run(&kw, &args, &configs)).unwrap_or_else(|| mers.run(&kw, &args, &configs))
}

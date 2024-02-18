
fn main() {
    match zm::commandline::parse() {
        Ok(args) if !args.is_empty() => {
            println!("{}", args.join(" "));
        }
        Err(e) => {
            eprintln!("Zm: error: {e}");
            std::process::exit(1)
        },
        _ => {}
    }
}

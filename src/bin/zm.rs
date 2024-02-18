
fn main() {
    match zm::parse() {
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

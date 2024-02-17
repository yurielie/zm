
fn main() {
    let args = match zm::parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Zm: error: {e}");
            std::process::exit(1)
        },
    };
    if !args.is_empty() {
        println!("{}", args.join(" "));
    }
}

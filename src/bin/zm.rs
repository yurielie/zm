use zm::Zm;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let zm = match Zm::from_file("./zm.json") {
        Ok(z) => z,
        Err(e) => {
            eprintln!("Zm: error: {e}");
            std::process::exit(1)
        },
    };
    let args = match zm.parse_args(&args) {
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

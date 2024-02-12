use zm::cmdline::CommandLine;

fn main() -> anyhow::Result<()> {
    let mut ht = CommandLine::new();
    ht.add_word_to_word("-h", "--help");

    let args: Vec<_> = std::env::args().skip(1).collect();
    let mapped = ht.parse_args(&args)?;
    println!("Zm: {:?}", mapped);

    Ok(())
}

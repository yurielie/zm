
use zm::handler::HandlerTable;


fn main() -> anyhow::Result<()> {
    let mut ht = HandlerTable::new();
    ht.add_word_to_word("-h", "--help");
    
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mapped = ht.parse_args(&args)?;
    println!("Zm: {:?}", mapped);

    Ok(())
}


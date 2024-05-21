use std::path::PathBuf;

use clap::Parser;
use csv::{Trim, Writer};
use tx_engine::process;

#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}

fn main() -> testresult::TestResult {
    let args = Args::parse();

    let mut rdr = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(&args.input)?;
    let records = rdr.deserialize().collect::<Result<_, _>>()?;
    let output = process(records);
    let mut writer = Writer::from_writer(std::io::stdout());
    for record in output.iter() {
        writer.serialize(&record)?;
    }
    writer.flush()?;
    Ok(())
}

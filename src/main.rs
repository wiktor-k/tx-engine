use std::path::PathBuf;

use clap::Parser;
use csv::Writer;
use tx_engine::process;

#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}

fn main() -> testresult::TestResult {
    env_logger::init();

    let args = Args::parse();

    let output = process(args.input)?;

    let mut writer = Writer::from_writer(std::io::stdout());
    for record in output.into_values() {
        writer.serialize(&record)?;
    }
    writer.flush()?;
    Ok(())
}

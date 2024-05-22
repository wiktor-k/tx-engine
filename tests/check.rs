use std::path::PathBuf;

use rstest::rstest;
use testresult::TestResult;

#[rstest]
fn main(#[files("tests/test-cases/*.input.csv")] path: PathBuf) -> TestResult {
    use std::collections::HashMap;

    use csv::{Trim, Writer};
    use tx_engine::{process, Account, ClientId};

    let output = PathBuf::from(path.display().to_string().replace(".input.", ".output."));
    eprintln!("found path: {path:?} output: {output:?}");

    let mut rdr = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(output)?;

    let mut accounts = HashMap::<ClientId, Account>::new();
    for item in rdr.deserialize() {
        let item: Account = item?;
        accounts.entry(item.client).or_insert(item);
    }
    let output = process(path)?;

    // Try to serialize all records.
    // This test prevents subtle serialization issues from appearing at runtime.
    let mut writer = Writer::from_writer(vec![]);
    for record in output.values() {
        writer.serialize(record)?;
    }
    writer.flush()?;
    let records = writer.into_inner()?;
    eprintln!("Records:\n{}", String::from_utf8_lossy(&records));

    assert_eq!(
        output, accounts,
        "expected output must be equal with what process spits out"
    );
    Ok(())
}

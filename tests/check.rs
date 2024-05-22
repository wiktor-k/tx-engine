use std::path::PathBuf;

use rstest::rstest;
use testresult::TestResult;

#[rstest]
fn main(#[files("tests/test-cases/*.input.csv")] path: PathBuf) -> TestResult {
    use std::collections::HashMap;

    use csv::Trim;
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
    assert_eq!(
        output, accounts,
        "expected output must be equal with what process spits out"
    );
    Ok(())
}

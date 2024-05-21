use std::path::PathBuf;

use rstest::rstest;
use testresult::TestResult;

#[rstest]
fn main(#[files("tests/test-cases/*.input.csv")] path: PathBuf) -> TestResult {
    let output = PathBuf::from(path.display().to_string().replace(".input", "output"));
    eprintln!("found path: {path:?} output: {output:?}");
    Ok(())
}

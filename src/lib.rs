use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use csv::Trim;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

pub type ClientId = u16;

type TxId = u32;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub r#type: RecordType,
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Option<Decimal>,
}

#[derive(Debug, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct Output {
    pub client: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

//FIXME: Whitespaces and decimal precisions (up to four places past the decimal) must be accepted by your program.
//FIXME: You can assume a precision of four places past the decimal and should output values with the same level of precision.
pub fn process(file: PathBuf) -> testresult::TestResult<HashMap<ClientId, Output>> {
    let mut rdr = csv::ReaderBuilder::new().trim(Trim::All).from_path(file)?;

    let mut accounts = HashMap::<ClientId, Output>::new();
    let mut txns = HashMap::<TxId, Decimal>::new();
    let mut disputed: HashSet<TxId> = HashSet::new();
    for item in rdr.deserialize() {
        let item: Record = item?;
        let account = accounts.entry(item.client).or_insert_with(|| Output {
            client: item.client,
            ..Default::default()
        });
        match item.r#type {
            RecordType::Deposit => {
                let Some(amount) = item.amount else {
                    panic!("Deposit without amount");
                };
                account.available += amount;
                account.total += amount;
                txns.entry(item.tx).or_insert(amount);
            }
            RecordType::Withdrawal => {
                let Some(amount) = item.amount else {
                    panic!("Withdrawal without amount");
                };
                if account.available >= amount {
                    account.available -= amount;
                    account.total -= amount;
                    txns.entry(item.tx).or_insert(amount);
                } else {
                    // transaction failed
                }
            }
            RecordType::Dispute => {
                if let Some(amount) = txns.get(&item.tx) {
                    account.available -= amount;
                    account.held += amount;
                    disputed.insert(item.tx);
                }
                // total amount stays the same
                // FIXME: if this tx doesn't exist it should be ignored
            }
            RecordType::Resolve => {
                if let Some(amount) = txns.get(&item.tx) {
                    account.held -= amount;
                    account.available += amount;
                    disputed.remove(&item.tx);
                }
                // total amount stays the same
                // FIXME: If the tx specified doesn't exist, or the tx isn't under dispute, you can ignore the resolve and assume this is an error on our partner's side.
            }
            RecordType::Chargeback => {
                if let Some(amount) = txns.get(&item.tx) {
                    if disputed.contains(&item.tx) {
                        account.held -= amount;
                        account.total -= amount;
                        // assuming "frozen" means "locked == true"
                        account.locked = true;
                        disputed.remove(&item.tx);
                    }
                }
            }
        }
    }
    Ok(accounts)
}

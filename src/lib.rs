use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use csv::Trim;
use rust_decimal::Decimal;
use serde::{ser::SerializeStruct, Deserialize, Serialize};

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
    #[serde(rename = "type")]
    pub kind: RecordType,
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Option<Decimal>,
}

#[derive(Debug, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct Output {
    pub client: ClientId,
    #[serde(flatten)]
    pub amounts: Amounts,
    pub locked: bool,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Amounts {
    pub available: Decimal,
    pub held: Decimal,
}

impl Amounts {
    fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
    }

    fn withdraw(&mut self, amount: Decimal) -> bool {
        if self.available >= amount {
            self.available -= amount;
            true
        } else {
            false
        }
    }

    fn hold(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
    }

    fn release(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
    }
}

impl Serialize for Amounts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut x = serializer.serialize_struct("Amounts", 3)?;
        x.serialize_field("available", &self.available)?;
        x.serialize_field("held", &self.held)?;
        // total is always the sum of available and held
        let total = self.available + self.held;
        x.serialize_field("total", &total)?;
        x.end()
    }
}

pub fn process(file: PathBuf) -> testresult::TestResult<HashMap<ClientId, Output>> {
    let mut rdr = csv::ReaderBuilder::new().trim(Trim::All).from_path(file)?;

    let mut accounts = HashMap::<ClientId, Output>::new();
    let mut txns = HashMap::<TxId, Decimal>::new();
    let mut disputed: HashSet<TxId> = HashSet::new();
    for record in rdr.deserialize() {
        let record: Record = record?;
        let account = accounts.entry(record.client).or_insert_with(|| Output {
            client: record.client,
            ..Default::default()
        });
        match record.kind {
            RecordType::Deposit => {
                let Some(amount) = record.amount else {
                    panic!("Deposit without amount");
                };
                account.amounts.deposit(amount);
                txns.entry(record.tx).or_insert(amount);
            }
            RecordType::Withdrawal => {
                let Some(amount) = record.amount else {
                    panic!("Withdrawal without amount");
                };
                if account.amounts.withdraw(amount) {
                    txns.entry(record.tx).or_insert(amount);
                } else {
                    // transaction failed
                }
            }
            RecordType::Dispute => {
                if let Some(amount) = txns.get(&record.tx) {
                    account.amounts.hold(*amount);
                    disputed.insert(record.tx);
                }
            }
            RecordType::Resolve => {
                if let Some(amount) = txns.get(&record.tx) {
                    account.amounts.release(*amount);
                    disputed.remove(&record.tx);
                }
            }
            RecordType::Chargeback => {
                if let Some(amount) = txns.get(&record.tx) {
                    if disputed.contains(&record.tx) {
                        account.amounts.chargeback(*amount);
                        // "frozen" means "locked == true"
                        account.locked = true;
                        disputed.remove(&record.tx);
                    }
                }
            }
        }
    }
    Ok(accounts)
}

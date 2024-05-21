use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use csv::{Trim, Writer};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RecordType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

type ClientId = u16;

type TxId = u32;

#[derive(Debug, Deserialize)]
pub struct Record {
    r#type: RecordType,
    client: ClientId,
    tx: TxId,
    amount: Option<Decimal>,
}

#[derive(Debug, Serialize, Default)]
pub struct Output {
    client: ClientId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

//FIXME: Whitespaces and decimal precisions (up to four places past the decimal) must be accepted by your program.
//FIXME: You can assume a precision of four places past the decimal and should output values with the same level of precision.
pub fn process(items: Vec<Record>) -> Vec<Output> {
    let mut accounts = HashMap::<ClientId, Output>::new();
    let mut txns = HashMap::<TxId, Decimal>::new();
    for item in items {
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
                }
                // total amount stays the same
                // FIXME: if this tx doesn't exist it should be ignored
            }
            RecordType::Resolve => {
                if let Some(amount) = txns.get(&item.tx) {
                    account.held -= amount;
                    account.available += amount;
                }
                // total amount stays the same
                // FIXME: If the tx specified doesn't exist, or the tx isn't under dispute, you can ignore the resolve and assume this is an error on our partner's side.
            }
            RecordType::Chargeback => {
                todo!();
            }
        }
    }
    accounts.into_values().collect()
}

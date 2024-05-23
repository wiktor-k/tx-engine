#![doc = include_str!("../README.md")]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use csv::Trim;
use rust_decimal::Decimal;
use serde::{ser::SerializeStruct, Deserialize, Serialize};

/// Represents a type of a record.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordType {
    /// Money deposit. Increases the available amount.
    Deposit,

    /// Money withdrawal. Decreases the available amount.
    Withdrawal,

    /// Transaction dispute. Moves funds from available to held.
    Dispute,

    /// Dispute resolution. Moves funds from held to available.
    Resolve,

    /// Chargeback. Freezes the account and decreases held funds.
    Chargeback,
}

/// Represents client identifier.
pub type ClientId = u16;

/// Represents transaction identifier.
pub type TxId = u32;

/// Transaction engine error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Deposit used but no amount has been specified.
    #[error("Deposit used but no amount is specified in transaction {0}")]
    DepositNoAmount(TxId),

    /// Withdraw used but no amount has been specified.
    #[error("Withdraw used but no amount is specified in transaction {0}")]
    WithdrawNoAmount(TxId),

    /// CSV serialization error.
    #[error("CSV serialization error: {0}")]
    Csv(#[from] csv::Error),
}

/// Result of transaction engine.
pub type Result<T> = std::result::Result<T, Error>;

/// Single record.
#[derive(Debug, Deserialize)]
pub struct Record {
    /// Type of the record.
    #[serde(rename = "type")]
    pub kind: RecordType,

    /// Identifies client account.
    pub client: ClientId,

    /// Specifies transaction identifier. For example deposits and
    /// withdrawals can be referenced by disputes.
    pub tx: TxId,

    /// The amount that this transaction represents. Note that only
    /// deposits and withdrawals will contain the amount. Other record
    /// types use the amount from referenced transactions.
    pub amount: Option<Decimal>,
}

/// Represents client account.
///
/// The account has associated funds stored in the `amounts` field and
/// can be frozen (`locked`).
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Account {
    /// Identifier of this account.
    pub client: ClientId,

    /// Funds associated with this account.
    #[serde(flatten)]
    pub amounts: Amounts,

    /// Frozen status of this account. The account is only frozen if a
    /// successful chargeback occurs.
    pub locked: bool,
}

impl Serialize for Account {
    /// Serializes account. The inner amounts (available and held) are serialized as
    /// usual. Total is added as a computed field.
    /// Sadly, #[serde(flatten)] is not supported by the "rust-csv" create, see:
    /// <https://github.com/BurntSushi/rust-csv/pull/223>
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut x = serializer.serialize_struct("Account", 3)?;
        x.serialize_field("client", &self.client)?;
        x.serialize_field("available", &self.amounts.available)?;
        x.serialize_field("held", &self.amounts.held)?;
        // total is always the sum of available and held
        x.serialize_field("total", &self.amounts.total())?;
        x.serialize_field("locked", &self.locked)?;
        x.end()
    }
}

/// Funds associated with the account.
///
/// The funds are split into two buckets:
///    - available - funds that the client can use in their transactions,
///    - held - funds that are held because of pending disputes.
///
/// Additionally there's a total getter which is a sum of the previous two.
#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Amounts {
    /// Funds that the client can use in transactions.
    pub available: Decimal,

    /// Funds that are blocked because of pending disputes.
    pub held: Decimal,
}

impl Amounts {
    /// Deposits new funds which increases the available amount.
    pub fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
    }

    /// Withdraws funds which decreases the available amount.
    ///
    /// Note that if the withdrawing amount is bigger than the
    /// available funds the operation is a no-op.  This function
    /// returns `true` on success and `false` on failure.
    pub fn withdraw(&mut self, amount: Decimal) -> bool {
        if self.available >= amount {
            self.available -= amount;
            true
        } else {
            false
        }
    }

    /// Marks a certain amount of funds as held for dispute.
    ///
    /// Decreases the available amount and increases the held amount
    /// by the same value.
    pub fn hold(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
    }

    /// Releases funds previously held for dispute.
    ///
    /// Decreases the held amount and increases the available amount.
    pub fn release(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    /// Completes the chargeback procedure removing held funds from this account.
    pub fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
    }

    /// Returns a total amount which is a sum of held and available funds.
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }
}

/// Process the input CSV file.
///
/// The input file will have the values stripped of whitespace.
pub fn process(file: impl AsRef<Path>) -> Result<HashMap<ClientId, Account>> {
    let mut rdr = csv::ReaderBuilder::new().trim(Trim::All).from_path(file)?;

    let mut accounts = HashMap::<ClientId, Account>::new();
    let mut txns = HashMap::<TxId, Decimal>::new();
    let mut disputed: HashSet<TxId> = HashSet::new();
    for record in rdr.deserialize() {
        let record: Record = record?;
        let account = accounts.entry(record.client).or_insert_with(|| Account {
            client: record.client,
            ..Default::default()
        });
        match record.kind {
            RecordType::Deposit => {
                let Some(amount) = record.amount else {
                    return Err(Error::DepositNoAmount(record.tx));
                };
                account.amounts.deposit(amount);
                txns.entry(record.tx).or_insert(amount);
            }
            RecordType::Withdrawal => {
                let Some(amount) = record.amount else {
                    return Err(Error::WithdrawNoAmount(record.tx));
                };
                if account.amounts.withdraw(amount) {
                    txns.entry(record.tx).or_insert(amount);
                } else {
                    log::info!("Transaction {} failed - insufficient funds.", record.tx);
                }
            }
            RecordType::Dispute => {
                if let Some(amount) = txns.get(&record.tx) {
                    account.amounts.hold(*amount);
                    disputed.insert(record.tx);
                } else {
                    log::info!("Dispute failed - transaction {} not found.", record.tx);
                }
            }
            RecordType::Resolve => {
                if let Some(amount) = txns.get(&record.tx) {
                    account.amounts.release(*amount);
                    disputed.remove(&record.tx);
                } else {
                    log::info!("Resolve failed - transaction {} not found.", record.tx);
                }
            }
            RecordType::Chargeback => {
                if let Some(amount) = txns.get(&record.tx) {
                    if disputed.contains(&record.tx) {
                        account.amounts.chargeback(*amount);
                        // "frozen" means "locked == true"
                        account.locked = true;
                        disputed.remove(&record.tx);
                    } else {
                        log::info!(
                            "Chargeback failed - transaction {} not under dispute.",
                            record.tx
                        );
                    }
                } else {
                    log::info!("Chargeback failed - transaction {} not found.", record.tx);
                }
            }
        }
    }
    Ok(accounts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn withdraw_ok() {
        let mut a = Amounts::default();
        a.deposit(2.into());
        assert!(a.withdraw(1.into()));
        assert_eq!(a.available, 1.into());
        assert_eq!(a.held, 0.into());
    }

    #[test]
    fn withdraw_failed() {
        let mut a = Amounts::default();
        a.deposit(1.into());
        assert!(!a.withdraw(2.into()));
        assert_eq!(a.available, 1.into());
        assert_eq!(a.held, 0.into());
    }
}

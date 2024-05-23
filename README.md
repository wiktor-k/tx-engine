# Simple transactional engine

[![CI](https://github.com/wiktor-k/tx-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/wiktor-k/tx-engine/actions/workflows/rust.yml)

This project implements a simple transactional engine where, in addition to standard deposits and withdrawal there are also disputes that can be resolved or charged-back.

Each client's account is modelled to contain not a single value but is split into several:
  - available - which is the sum that the client can use,
  - held - amount that is blocked as there are transactions that are disputed,
  - total - sum of available and held.

## Running

The crate exposes a command-line interface, where the first argument is a filename to process. It will output the state of accounts in CSV format:

```sh
$ cargo run -- tests/test-cases/chargeback-ok.input.csv
client,available,held,total,locked
1,0,0,0,true
```

Additionally, it can be used as a library. The engine exposes `process` function:

```rust
use tx_engine::process;

let results = process("tests/test-cases/chargeback-ok.input.csv").expect("processing to succeed");
```

## Supported transaction types

The project implements several test-cases based on the specification (see `tests/test-cases` directory). The exact test case name will be inserted in `code` below.

As a general rule the engine strips whitespace (`with-spaces`) and uses decimals for handling amounts (`four-decimal-places`).

### Deposit

Increases the amount that is available. As total is influenced by the available amount it is also increased. (`plain`)

### Withdraw

Decreases the amount that is available. As total is influenced by the available amount it is also decreased. (`plain`)

If the withdrawal would make the available negative it is ignored. (`withdrawal-no-sufficient-funds`)

### Dispute

The transaction that is referenced by the dispute makes the client's available sum decreased by the amount that is in the transaction. These funds are now stored in the held field. (`dispute-ok`)

If the dispute references a non-existent transaction it is ignored. (`dispute-bad-tx`)

### Resolve

Marks the dispute as resolved effectively reversing the action of dispute. (`resolve-ok`)

If the resolve references a non-existent transaction it is ignored. If it references a transaction that is not being disputed it's also ignored. (`resolve-bad-tx`)

### Chargeback

Marks the dispute as resolved reversing the underlying transaction that was under the dispute. (`chargeback-ok`)

If the chargeback references a non-existent transaction it is ignored (`chargeback-bad-tx`). If it references a transaction that is not being disputed it's also ignored (`chargeback-not-disputed`).

There's additional test which chargebacks one transaction that is disputed out of two that are open (`chargeback-disputed-and-not-disputed`).

## Open questions

1. The dispute for both withdrawals and deposits is handled the same way. Should it be handled differently? (Because deposits are basically the reverse of a withdrawal)

2. The library uses decimals for arbitrary precision due to correctness. It could be optimized using the "only four places are required".

## Future work

One are of improvement that could be pursued is converting operations (deposits, withdrawals) into enums which take appropriate values. This way we'd avoid the optional amount (that doesn't exist for disputes) and the [values would always be valid](https://fsharpforfunandprofit.com/posts/designing-with-types-making-illegal-states-unrepresentable/).

Unfortunately [rust-csv doesn't support](https://github.com/BurntSushi/rust-csv/pull/231) [tagged enums](https://serde.rs/enum-representations.html).

This could be worked around with a custom Deserializer (a proof-of-concept is in the `tagged-enums` branch).

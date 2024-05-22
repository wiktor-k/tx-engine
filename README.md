# Simple transactional engine

[![CI](https://github.com/wiktor-k/tx-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/wiktor-k/tx-engine/actions/workflows/rust.yml)

This project implements a simple transactional engine where, in addition to standard deposits and withdrawal there are also disputes that can be resolved or charged-back.

Each client's account is modelled to contain not a single value but is split into several:
  - available - which is the sum that the client can use,
  - held - amount that is blocked as there are transactions that are disputed,
  - total - sum of available and held.

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

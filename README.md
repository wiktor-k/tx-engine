# Simple transactional engine

This project implements a simple transactional engine where, in addition to standard deposits and withdrawal there are also disputes that can be resolved or charged-back.

Each client's account is modelled to contain not a single value but is split into several:
  - available - which is the sum that the client can use,
  - held - amount that is blocked as there are transactions that are disputed,
  - total - sum of available and held.

## Supported transaction types

### Deposit

Increases the amount that is available. As total is influenced by the available amount it is also increased.

### Withdraw

Decreases the amount that is available. As total is influenced by the available amount it is also decreased.

If the withdrawal would make the available negative it is ignored.

### Dispute

The transaction that is referenced by the dispute makes the client's available sum decreased by the amount that is in the transaction. These funds are now stored in the held field.

If the dispute references a non-existent transaction it is ignored.

### Resolve

Marks the dispute as resolved effectively reversing the action of dispute.

If the resolve references a non-existent transaction it is ignored. If it references a transaction that is not being disputed it's also ignored.

### Chargeback

Marks the dispute as resolved reversing the underlying transaction that was under the dispute.

If the chargeback references a non-existent transaction it is ignored. If it references a transaction that is not being disputed it's also ignored.

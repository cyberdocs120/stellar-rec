# 📅 5-Day Intensive Development Plan: stellar-rec Protocol

This plan outlines a 5-day "Sprint to 65%" aimed at completing the core protocol logic for the `stellar-rec` workspace. By the end of this sprint, the system will have a hardened Oracle layer, a functional CfD engine, and a verified Retirement registry.

## 📈 Completion Status

- **Current:** ~62% (Day 3 Complete: Oracle Hardening & CfD Engine implemented)
- **Target:** 65%+ (Phase 0-4 core logic implemented and integration-tested)

---

## 🛠 Daily Breakdown & Prompts

### Day 1: Oracle Handler — Economic Security & Slashing

**Goal:** Transition the Oracle from a simple data relay to an economically secured network.

- **Tasks:**
  - Implement Oracle node staking (bond requirement).
  - Implement slashing logic in `resolve_dispute`.
  - Implement fee distribution to honest Oracle nodes.
  - Add reputation tracking to `OracleNode` storage.

> **Prompt for Day 1:**
> "Hardening the Oracle Handler: Implement an Oracle Staking mechanism. Oracle nodes must now post a collateral bond to be 'Active'. Enhance the `resolve_dispute` function to slash the bonds of all nodes that signed a reading found to be fraudulent. Implement a `claim_rewards` function for Oracles, funded by a 15% protocol fee from minting/trading. Update `storage.rs` and `types.rs` to track node stakes, accumulated rewards, and reputation scores. Ensure all logic aligns with the security mandates in README.md."

---

### Day 2: Marketplace — CfD Infrastructure & Collateral

**Goal:** Lay the groundwork for the Contract-for-Difference (CfD) engine.

- **Tasks:**
  - Define `CfDPosition` and `PositionState` in `types.rs`.
  - Implement `open_cfd` and `accept_cfd`.
  - Implement atomic collateral escrow for yUSDC.
  - Implement `add_collateral` / `remove_collateral` functions.

> **Prompt for Day 2:**
> "Marketplace CfD Foundations: Implement the storage and initialization logic for Contracts-for-Difference. Define the `CfDPosition` structure in `types.rs` and update `storage.rs` for persistent position tracking. Implement `open_cfd` to allow traders to propose a hedge (strike, qty, expiry) and `accept_cfd` for counterparties. Both must handle atomic yUSDC transfers from the traders to the Marketplace vault as initial margin. Add margin management functions to allow parties to adjust their collateral levels before settlement."

---

### Day 3: Marketplace — CfD Engine & Settlement Logic

**Goal:** Complete the mathematical engine for the CfD derivative.

- **Tasks:**
  - Implement Mark-to-Market (MtM) price fetching.
  - Implement the `settle_cfd` payoff formula.
  - Implement `liquidate` for under-collateralized positions.
  - Add event emissions for all CfD lifecycle stages.

> **Prompt for Day 3:**
> "CfD Engine Completion: Implement the settlement and liquidation logic for CfDs in the Marketplace contract. Create the `settle_cfd` function that fetches the current reference price from the Oracle Handler and executes the payoff transfer between parties based on the strike price. Implement a `liquidate` function that allows the protocol (or a bot) to close positions where the collateral has dropped below the maintenance margin (10% by default). Ensure the math matches the 'Contract-for-Difference (CfD) Logic' section in the README.md and uses safe arithmetic."

---

### Day 4: Retirement Registry — Atomic Burn & Proofs

**Goal:** Implement the final stage of the REC lifecycle.

- **Tasks:**
  - Implement the `retire` function (burns REC and records receipt).
  - Create the `RetirementReceipt` storage and unique ID generation.
  - Implement `verify_retirement` for third-party auditors.
  - Implement cross-contract call to `rec-token::burn`.

> **Prompt for Day 4:**
> "Retirement Registry Implementation: Build the core logic for the Retirement contract. Implement the `retire` function which accepts a list of REC tokens, verifies ownership, and performs a cross-contract call to the REC Token's `burn` function. The contract must generate and store a `RetirementReceipt` containing the claim details (period, purpose, jurisdiction) and emit a `RecRetired` event. Implement the `verify_retirement` query to allow auditors to check the status of any token ID. Ensure the retirement logic is immutable and prevents double-claiming."

---

### Day 5: Integration & Verification — End-to-End Sandbox

**Goal:** Validate the entire protocol lifecycle and ensure 100% architectural consistency.

- **Tasks:**
  - Write end-to-end integration tests in `tests/integration`.
  - Simulate edge cases (mismatched vintages, failed CfD liquidations).
  - Perform a workspace-wide error and event audit.
  - Final documentation update.

> **Prompt for Day 5:**
> "Protocol Integration & Lifecycle Validation: Create a comprehensive integration test suite in `tests/integration/src/lib.rs`. The tests must simulate a 100% complete lifecycle: 1) Oracle registration and meter binding. 2) Meter reading submission and REC minting. 3) Marketplace listing and spot trade settlement. 4) CfD hedging, price movement, and liquidation. 5) Final REC retirement and receipt verification. Verify that all contracts respect the `ContractPaused` state and that error codes across the workspace are consistent with the Event & Error Reference in README.md."

---

## 🚀 Post-Sprint Outcome

At the conclusion of Day 5, the `stellar-rec` project will have reached **65% completion**, representing a robust, functional, and economically secured foundation for the Renewable Energy Certificate market on Stellar.

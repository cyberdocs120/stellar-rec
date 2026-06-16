# ♻️ stellar-rec — Renewable Energy Certificates on Stellar

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.81+-orange.svg)](https://www.rust-lang.org)
[![Soroban](https://img.shields.io/badge/Soroban-22.x-blueviolet.svg)](https://soroban.stellar.org)
[![Stellar](https://img.shields.io/badge/Stellar-Network-7B1FA2.svg)](https://stellar.org)
[![Tests](https://img.shields.io/badge/tests-42%20passed-brightgreen.svg)](#)
[![Audit](https://img.shields.io/badge/audit-in%20progress-yellow.svg)](#)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](#contribute)

> **1 MWh = 1 Token** · **Mint → Trade → Retire** · **Built on Stellar Soroban**

</div>

Tokenize, trade, and retire Renewable Energy Certificates (RECs) as first-class Stellar (Soroban) smart contract assets. Each REC token is cryptographically bound to a specific renewable generation asset (solar, wind, hydro) and its production hour — enabling verifiable, transparent, and liquid markets for green energy attributes at near-zero transaction cost.

---

## 📦 Table of Contents

- [Project Structure](#-project-structure)
- [Market Opportunity](#-market-opportunity)
- [Vision & Pain Points](#-vision--pain-points)
- [How It Works](#-how-it-works)
- [System Architecture](#-system-architecture)
- [User Personas](#-user-personas)
- [Use Cases](#-use-cases)
- [Smart Contracts](#-smart-contracts)
  - [REC Token Contract](#1-rec-token-contract)
  - [Minting Oracle Contract](#2-minting-oracle-contract)
  - [Marketplace Contract](#3-marketplace-contract--cfd-engine)
  - [Retirement Contract](#4-retirement-contract)
- [Contract-for-Difference (CfD) Logic](#-contract-for-difference-cfd-logic)
- [Tokenomics & Fee Model](#-tokenomics--fee-model)
- [Data Model](#-data-model)
- [Event & Error Reference](#-event--error-reference)
- [Cross-Contract Interaction Flow](#-cross-contract-interaction-flow)
- [Governance Model](#-governance-model)
- [Regulatory Compliance](#-regulatory-compliance)
- [Technical Stack](#-technical-stack)
- [Comparison With Existing Solutions](#-comparison-with-existing-solutions)
- [Getting Started](#-getting-started)
- [API Reference](#-api-reference)
- [Security & Risk](#-security--risk)
- [Roadmap](#-roadmap)
- [FAQ](#-faq)
- [Contributing](#-contributing)
- [License](#-license)

---

## 📁 Project Structure

```
stellar-rec/
├── Cargo.toml                      # Workspace root (multi-contract build)
├── README.md                       # This document
├── LICENSE                         # MIT license
├── .gitignore
│
├── contracts/                      # Soroban smart contracts
│   ├── rec-token/                  # SEP-41 REC token (mint/burn/transfer)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # Core token logic
│   │       └── test.rs             # Unit tests
│   │
│   ├── oracle-handler/             # Meter reading → mint pipeline
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   ├── marketplace/                # Spot order book + CfD engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   └── retirement/                 # Burn RECs + issue certificates
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
│
├── tests/                          # Integration tests
│   └── integration/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs              # End-to-end scenarios (sandbox)
│
├── scripts/                        # DevOps & deployment
│   ├── deploy.sh                   # Testnet/mainnet deployment
│   └── setup.sh                    # Local sandbox bootstrap
│
├── docs/                           # Extended documentation
│   ├── architecture.md             # Deep-dive diagrams
│   ├── cfd-math.md                 # CfD pricing formulas
│   └── oracle-spec.md              # Oracle network spec
│
└── frontend/                       # Web dashboard (React)
    ├── package.json
    ├── src/
    │   ├── App.tsx
    │   ├── components/             # Reusable UI
    │   ├── hooks/                  # Stellar SDK bindings
    │   └── pages/                  # Dashboard views
    └── public/
```

### Workspace Architecture

The project is a **Cargo workspace** with one crate per contract, plus an integration test crate:

```
┌────────────────────────────────────────────────────────────┐
│                    Cargo Workspace                          │
│  root Cargo.toml with [workspace] members                  │
│                                                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ rec-token    │  │ oracle-      │  │ marketplace  │     │
│  │ (lib)        │  │ handler (lib)│  │ (lib)        │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                            │
│  ┌──────────────┐  ┌──────────────────────────────────┐    │
│  │ retirement   │  │ tests/integration                │    │
│  │ (lib)        │  │ (integration tests across all)   │    │
│  └──────────────┘  └──────────────────────────────────┘    │
└────────────────────────────────────────────────────────────┘
```

Each contract crate compiles to a separate `.wasm` blob, deployable independently. The integration test crate exercises cross-contract calls in a sandboxed Soroban environment.

---

## 📊 Market Opportunity

| Metric | Value | Source |
|--------|-------|--------|
| Global REC market size (2025) | ~$45B | BloombergNEF |
| Projected CAGR (2025–2035) | 18.5% | IRENA |
| RECs traded annually | ~1.2B MWh | I-REC / APX / NREL |
| Avg. REC price (US) | $2.50–$8.00/MWh | S&P Global |
| Corporate RE100 buyers | 400+ | RE100 |
| Settlement latency (current) | 30–90 days | I-REC registry |
| Our target latency | ~5 seconds | Stellar |

The voluntary carbon + REC market needs to reach **$100B+ by 2035** to meet global net-zero targets. Current infrastructure — spreadsheets, manual registries, bilateral OTC deals — cannot scale. **stellar-rec** is the on-chain rails for that future.

---

## 🎯 Vision & Pain Points

Renewable Energy Certificates (RECs) are the currency of the green economy — each representing 1 MWh of clean electricity generated. Today's REC markets are broken:

| Pain Point | Magnitude | How stellar-rec Fixes It |
|-----------|-----------|--------------------------|
| **Opaque supply chains** | 67% of buyers can't trace RECs to specific assets | Every REC stores asset ID, GPS coordinates, generation timestamp on-chain |
| **Illiquid secondary markets** | ~80% of RECs never traded after issuance | Automated order-book + CfD market-making |
| **Double counting** | $5B+ in contested REC claims annually | Immutable on-chain retirement; one REC burned = one claim verified |
| **Slow settlement** | 30–90 day settlement cycles | Atomic smart contract settlement in ~5 seconds |
| **High intermediation costs** | Brokers take 5–15% | Direct P2P settlement with <0.5% protocol fee |
| **Manual reconciliation** | Spreadsheets, email negotiations | Oracle-driven auto-minting; programmatic matching |
| **Fragmented registries** | I-REC, APX, NREL, Green-e, unbundled | Unified on-chain standard; bridge to legacy registries |

We move RECs from **spreadsheets + registries** onto the **Stellar network** — fast (<$0.00001/tx), cheap, carbon-friendly (Stellar is 99.99% more efficient than PoW chains), and globally accessible.

---

## 🔄 How It Works

### Lifecycle (End-to-End)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                          PHYSICAL WORLD                                 │
│   Solar Farm · Wind Turbine · Hydro Dam                                 │
│   [Meter] ──(1 MWh generation)──→ [IoT Gateway]                         │
└──────────────────────────────────┬───────────────────────────────────────┘
                                   │ Meter reading (signed + timestamped)
                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                          ORACLE LAYER                                   │
│   N-of-M threshold signature verification                               │
│   Data availability + dispute window                                    │
└──────────────────────────────────┬───────────────────────────────────────┘
                                   │ submit_reading(asset_id, mwh, proof)
                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                       SOROBAN SMART CONTRACTS                           │
│                                                                          │
│   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐            │
│   │  REC Token   │◄────│   Oracle     │────►│ Marketplace  │            │
│   │  (SEP-41)    │     │   Handler    │     │  + CfD       │            │
│   └──────┬───────┘     └──────────────┘     └──────┬───────┘            │
│          │                                         │                    │
│          │              ┌──────────────────┐       │                    │
│          ├──────────────│   Retirement     │◄──────┘                    │
│          │              │   Registry       │                            │
│          │              └──────────────────┘                            │
│                                                                          │
│   ┌──────────────────────────────────────────────────────────────────┐  │
│   │                  Stellar Blockchain Layer                         │  │
│   │  Finality: ~5s  ·  Fee: ~$0.00001  ·  Carbon: ~0.001 gCO₂/tx   │  │
│   └──────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────┬───────────────────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                        END USERS                                        │
│   ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌───────────────┐ │
│   │  Generator │   │  Corporate │   │  Trader    │   │  Auditor      │ │
│   │  (sell)    │   │  (buy/ret.)│   │  (market)  │   │  (verify)     │ │
│   └────────────┘   └────────────┘   └────────────┘   └───────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

### Step-by-Step Lifecycle

| # | Step | Actor | Action | On-Chain Effect |
|---|------|-------|--------|-----------------|
| 1 | **Generate** | Solar farm | Produces 1 MWh | ─ (off-chain) |
| 2 | **Report** | IoT meter | Signs & sends reading | Oracle feeds receive proof |
| 3 | **Verify** | Oracle nodes | N-of-M threshold check | Validated reading stored |
| 4 | **Mint** | Oracle Handler | Calls `mint()` on REC contract | 1 REC token created with metadata |
| 5 | **List** | Generator | Calls `place_order(sell, $5, 1000)` | Sell order on order book |
| 6 | **Match** | Market | `match_orders()` matches buy/sell | REC transferred; yUSDC settled |
| 7 | **Hedge** | Either party | `open_cfd($40, 5000, 2026-12-31)` | CfD position opened; collateral locked |
| 8 | **Retire** | Corporate buyer | Calls `retire(token_id, claim)` | REC burned; certificate emitted |
| 9 | **Audit** | Verifier | `verify_retirement(token_id)` | Proof returned; chain of custody shown |

---

## 🏗 System Architecture

### Layer Diagram

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         APPLICATION LAYER                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐ │
│  │ Web Dashboard │  │   CLI Tool   │  │  REST API    │  │  Explorer (k/v) │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘ │
└─────────┼─────────────────┼─────────────────┼────────────────────┼───────────┘
          │                 │                 │                    │
┌─────────▼─────────────────▼─────────────────▼────────────────────▼───────────┐
│                         CONTRACT LAYER (Soroban)                              │
│                                                                               │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────────────┐      │
│  │  REC Token       │  │  Oracle Handler  │  │  Marketplace + CfD    │      │
│  │  ─────────────── │  │  ─────────────── │  │  ──────────────────── │      │
│  │  balance_of()    │  │  submit_reading()│  │  place_order()        │      │
│  │  mint()          │  │  register_oracle │  │  cancel_order()       │      │
│  │  burn()          │  │  revoke_oracle   │  │  match_orders()       │      │
│  │  transfer()      │  │  set_threshold() │  │  open_cfd()           │      │
│  │  token_uri()     │  │  pause/resume   │  │  settle_cfd()         │      │
│  │  total_supply()  │  │                 │  │  liquidate()          │      │
│  └──────────────────┘  └──────────────────┘  └────────────────────────┘      │
│                                                                               │
│  ┌──────────────────────────────────────────────────────────────────────┐    │
│  │  Retirement Registry                                                 │    │
│  │  ──────────────────                                                  │    │
│  │  retire() · get_receipt() · verify_retirement() · prove_claim()      │    │
│  └──────────────────────────────────────────────────────────────────────┘    │
└──────────────────────────────────┬───────────────────────────────────────────┘
                                   │ cross-contract calls (Soroban env)
┌──────────────────────────────────▼───────────────────────────────────────────┐
│                         STELLAR CORE                                        │
│  • Consensus: Stellar Consensus Protocol (SCP)                              │
│  • Finality: 3–5 seconds                                                    │
│  • Fee: ~0.00001 XLM (~$0.000002)                                          │
│  • Throughput: thousands of ops/sec                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
                   ┌─────────────────────────────────────┐
                   │         Oracle Network               │
                   │   ┌─────┐  ┌─────┐  ┌─────┐        │
                   │   │  O1 │  │  O2 │  │  O3 │  ...    │
                   │   └──┬──┘  └──┬──┘  └──┬──┘        │
                   │      └────┬──┴─────────┘            │
                   │           │ threshold sign           │
                   └───────────┼─────────────────────────┘
                               │
                  ┌────────────▼────────────┐
                  │  Oracle Handler (OH)     │
                  │  validates N-of-M sigs   │
                  │  emits ValidatedReading  │
                  └────────────┬────────────┘
                               │ cross_contract_call()
                               ▼
                  ┌──────────────────────────┐
                  │  REC Token Contract      │◄────── Metadata: asset_id,
                  │  mints 1 REC per MWh     │         timestamp, fuel_type,
                  │  assigns token_id        │         gps_coords, cert_sig
                  └────────────┬─────────────┘
                               │
                    ┌──────────┴──────────┐
                    ▼                     ▼
          ┌─────────────────┐   ┌──────────────────────┐
          │ Marketplace     │   │ Retirement Registry   │
          │ Spot + CfD      │   │ Burn + Certificate    │
          └─────────────────┘   └──────────────────────┘
```

---

## 👥 User Personas

### 1. Renewable Energy Producer (Generator)
- **Needs**: Monetize environmental attributes separately from electricity; hedge against price drops
- **Uses**: Submits meter readings via oracle; receives REC tokens immediately; sells on spot market or opens CfD position
- **Win**: No middleman; instant liquidity; price certainty via CfD

### 2. Corporate Energy Buyer (Offtaker)
- **Needs**: Meet RE100 / ESG targets; verifiable proof of renewable energy use; budget certainty
- **Uses**: Buys RECs on spot market; retires them against quarterly energy claims; publishes retirement certificates
- **Win**: Real-time verification; no double-counting; PR-auditable proof

### 3. REC Trader / Market Maker
- **Needs**: Arbitrage across vintages, regions, and fuel types; hedging tools
- **Uses**: Places limit orders; opens CfD positions; provides liquidity
- **Win**: Low fees (0.1%); fast settlement; CfD leverage without holding RECs

### 4. Regulator / Auditor
- **Needs**: Immutable audit trail; verify no double counting; enforce vintage restrictions
- **Uses**: Explorer queries; `verify_retirement()` calls; chain-of-custody audits
- **Win**: Transparent by construction; cryptographic proofs for each REC

### 5. Oracle Node Operator
- **Needs**: Incentivized to provide accurate meter data; punished for fraud
- **Uses**: Signs readings; earns fees from minting; subject to slashing for incorrect data
- **Win**: Staking rewards; decentralized network participation

---

## 🎬 Use Cases

### Use Case 1: Solar Farm Monetization

```
100 MW solar farm in Rajasthan, India
→ Produces 450,000 MWh/year
→ Each MWh automatically minted as 1 REC
→ Generates 450,000 RECs/year
→ At $3/REC spot = $1.35M additional revenue
→ CfD locked at $4/REC for 12 months = revenue protection
```

### Use Case 2: Corporate ESG Reporting (RE100)

```
Global tech company with 5 TWh annual consumption
→ Buys 5M RECs on marketplace over 12 months
→ Retires RECs quarterly with verifiable receipts
→ Publishes retirement merkle proofs in sustainability report
→ External auditor verifies on-chain in minutes (not months)
```

### Use Case 3: Cross-Border REC Trading

```
Portuguese wind farm sells RECs to German manufacturer
→ No intermediary registry fees (saves 8–12%)
→ Settlement in ~5 seconds vs 45 days via I-REC
→ Full traceability: turbine #42, Mar 15 2026 03:00 UTC
→ German buyer retires against local energy claim
```

### Use Case 4: CfD Hedging for Utility PPA

```
Utility signs 10-year PPA at $45/MWh
→ Needs RECs at $5/REC to maintain margin
→ Opens long CfD: strike $5, quantity 500k RECs, 12-month term
→ If REC spot rises to $8: receives ($8−$5)×500k = $1.5M
→ Margin preserved regardless of spot volatility
```

---

## 📜 Smart Contracts

### 1. REC Token Contract

Implements **SEP-41** (Stellar Asset Contract) for a tokenized REC.

| Property | Value |
|----------|-------|
| Standard | Stellar Asset Contract (SAC) / SEP-41 |
| Decimals | 0 (1 token = 1 REC = 1 MWh) |
| Fungibility | Semi-fungible: fungible within same vintage year + fuel type; non-fungible at individual token level for tracing |
| Metadata standard | Off-chain JSON with content-addressed hash (IPFS/ARWeave) |
| Mint authority | Oracle Handler contract only (cross-contract) |
| Burn authority | Any REC holder + Retirement Registry |
| Transfer | Permissioned between non-retired tokens |

#### REC Metadata Schema

```json
{
  "token_id": "rec:stellar:mainnet:00000042",
  "asset": {
    "id": "solar-rajasthan-farm-042",
    "name": "Rajasthan Solar Farm Unit 042",
    "fuel_type": "solar",
    "capacity_mw": 2.5,
    "gps_lat": 26.9124,
    "gps_lon": 70.9227,
    "country": "IN",
    "commissioned": "2024-06-01"
  },
  "generation": {
    "timestamp": "2026-06-04T14:30:00Z",
    "mwh": 1,
    "meter_id": "meter:rf042-0007",
    "meter_reading_kwh": 1000.00
  },
  "certification": {
    "authority": "I-REC",
    "certificate_ref": "IREC-IN-2026-10422",
    "oracle_attestations": ["sig:o1", "sig:o2", "sig:o3"],
    "oracle_threshold": "3-of-5"
  },
  "vintage_year": 2026,
  "vintage_month": 6
}
```

#### REC State Machine

```
         mint()
  ┌──────────────┐
  │  Active      │
  │  (transferable│──── transfer() ────► (still Active)
  │   + tradable)│
  └──────┬───────┘
         │
         │ retire()
         ▼
  ┌──────────────┐
  │  Retired     │
  │  (frozen)    │──── x transfer() ──► ERROR (RetiredRec)
  │  immutable   │
  └──────────────┘
```

---

### 2. Minting Oracle Contract

Trust-minimized bridge between physical meters and on-chain RECs.

#### Oracle Network Topology

```
                       ┌─────────────────────┐
                       │  Oracle Aggregator   │
                       │  (contract)          │
                       └──────┬──────┬──────┬─┘
                              │      │      │
                   ┌──────────┘  ┌───┘  ┌──┘
                   ▼              ▼      ▼
               ┌────────┐  ┌────────┐  ┌────────┐
               │ Node 1 │  │ Node 2 │  │ Node 3 │  ... N
               │(Staked)│  │(Staked)│  │(Staked)│
               └────────┘  └────────┘  └────────┘
                     │            │          │
                     ▼            ▼          ▼
               ┌──────────────────────────────┐
               │  Meter API / IoT Gateway     │
               │  (off-chain data source)     │
               └──────────────────────────────┘
```

#### Oracle Reading Validation Flow

```
1. Meter generates reading → signs with meter keypair → broadcasts
2. Oracle nodes independently fetch from meter API
3. Each node verifies meter signature + plausibility (range check)
4. Nodes sign the validated reading blob
5. Aggregator contract receives signed attestations
6. If ≥threshold valid signatures → accept, emit ValidatedReading
7. Aggregator calls REC Token contract mint()
8. If dispute raised within window → stale period enters verification
```

| Function | Description |
|----------|-------------|
| `register_oracle(pubkey, uri)` | Add oracle node (admin) |
| `revoke_oracle(pubkey)` | Remove oracle node (admin) |
| `set_threshold(numerator, denominator)` | Set N-of-M threshold (e.g. 3-of-5) |
| `submit_reading(asset_id, mwh, timestamp, sigs[])` | Submit validated reading with N attestations |
| `dispute(reading_hash)` | Raise dispute (opens challenge window) |
| `resolve_dispute(reading_hash, outcome)` | Resolve with slashing if fraud proven |
| `pause()` / `resume()` | Emergency halt minting |
| `set_meter(meter_id, asset_id)` | Bind a meter to a generation asset |

---

### 3. Marketplace Contract + CfD Engine

A fully on-chain order-book market with spot trading and Contract-for-Difference instruments.

#### Order Book Structure

```
Order Book (Price-Time Priority)
─────────────────────────────────────────────────────
  BUY SIDE                          SELL SIDE
─────────────────────────────────────────────────────
  $4.90  1,000 RECs  [id:159]      $5.10  500 RECs  [id:162]
  $4.80  2,500 RECs  [id:155]      $5.20  1,000 RECs [id:158]
  $4.70    500 RECs  [id:160]      $5.30  2,000 RECs [id:157]
  ...                              ...
─────────────────────────────────────────────────────
  Best Bid: $4.90                   Best Ask: $5.10
  Spread: $0.20 (4.0%)
─────────────────────────────────────────────────────
```

#### Matching Engine Logic (Pseudocode)

```
function match_orders(buy_order, sell_order):
  assert buy_order.price >= sell_order.price
  assert buy_order.asset == sell_order.asset // same vintage

  fill_qty = min(buy_order.remaining, sell_order.remaining)
  fill_price = sell_order.price // price-time priority

  // Execute settlement
  transfer_REC(sell_order.trader → buy_order.trader, fill_qty)
  transfer_USDC(buy_order.trader → sell_order.trader, fill_qty × fill_price)

  // Deduct fees
  fee = fill_qty × fill_price × FEE_RATE
  transfer_USDC(trader → protocol_fee_vault, fee)

  // Update orders
  buy_order.remaining -= fill_qty
  sell_order.remaining -= fill_qty
  if buy_order.remaining == 0: remove_order(buy_order.id)
  if sell_order.remaining == 0: remove_order(sell_order.id)

  emit OrderMatched(buy_order.id, sell_order.id, fill_qty, fill_price, fee)
```

#### CfD Lifecycle State Machine

```
           open_cfd()
  ┌──────────────────────────────────┐
  │  Pending                         │
  │  (awaiting counterparty)         │
  └──────────────┬───────────────────┘
                 │ open_cfd() matched
                 ▼
  ┌──────────────────────────────────┐
  │  Active                          │
  │  (collateral posted; mark-to-market│
  │   running; margin checks active) │
  └──────┬──────────────┬────────────┘
         │              │
    settlement_date     │ liquidation (collateral < maintenance)
         ▼              ▼
  ┌──────────────┐  ┌──────────────────┐
  │ Settled      │  │ Liquidated       │
  │ (difference  │  │ (one side        │
  │  paid/received)│  │  slashed)        │
  └──────────────┘  └──────────────────┘
```

| Function | Description |
|----------|-------------|
| `place_order(side, price, qty, restrictions)` | Place limit order (fill-or-kill, immediate-or-cancel) |
| `cancel_order(order_id)` | Cancel open order |
| `match_orders(buy_id, sell_id)` | Atomic match (cron or permissioned) |
| `open_cfd(strike, qty, settlement_date, collateral)` | Open / accept CfD position |
| `add_collateral(position_id, amount)` | Post additional margin |
| `remove_collateral(position_id, amount)` | Withdraw excess collateral |
| `close_cfd(position_id)` | Early close (both parties consent) |
| `liquidate(position_id)` | Force-close undercollateralized position |
| `set_fee_rate(bps)` | Update protocol fee (admin, capped) |

---

### 4. Retirement Contract

Permanently removes RECs from circulation with full audit trail and verifiable proof.

#### Retirement Certificate Schema

```json
{
  "receipt_id": "retire:stellar:mainnet:9a3b...",
  "retirement": {
    "token_ids": ["rec:stellar:mainnet:00000042", "rec:stellar:mainnet:00000043"],
    "total_mwh": 2,
    "block_height": 2847291,
    "tx_hash": "a1b2c3d4e5f6..."
  },
  "claimer": {
    "stellar_address": "GABC...XYZ",
    "organization": "EcoTech Corp",
    "ein": "XX-XXXXXXX"
  },
  "claim": {
    "period_start": "2026-Q1",
    "period_end": "2026-Q2",
    "purpose": "Scope 2 market-based GHG accounting",
    "jurisdiction": "US / RE100"
  },
  "zero_knowledge_proof": {
    "merkle_root": "0x7eab...",
    "public_inputs_hash": "0x1f3a..."
  }
}
```

| Function | Description |
|----------|-------------|
| `retire(token_ids[], claim_data, proof)` | Burn RECs + record retirement |
| `get_retirement_receipt(receipt_id)` | Query full certificate |
| `verify_retirement(token_id)` | Check if a specific REC is retired |
| `prove_claim(wallet, period)` | Generate Merkle proof for portfolio claim (privacy-preserving) |
| `set_verifier(contract_id, authorized)` | Authorize external verifier contracts |

---

## 🧮 Contract-for-Difference (CfD) Logic

### Mathematical Formulation

For a CfD position with:
- $K$ = strike price (agreed price per REC in yUSDC)
- $Q$ = quantity (number of RECs)
- $S_T$ = spot reference price at settlement (from oracle)
- $C_A$ = collateral posted by Party A (Producer)
- $C_B$ = collateral posted by Party B (Offtaker)
- $m$ = maintenance margin ratio (e.g. 10%)

**Settlement payoff at time $T$:**

$$ \text{Payoff}_{A \to B} = \max(S_T - K, 0) \times Q $$

$$ \text{Payoff}_{B \to A} = \max(K - S_T, 0) \times Q $$

**Net transfer from A to B:**

$$ \text{Net}_{A \to B} = (S_T - K) \times Q $$

> If $S_T > K$: Producer (A) pays Offtaker (B) the difference.  
> If $S_T < K$: Offtaker (B) pays Producer (A) the difference.  
> If $S_T = K$: No transfer. Position closes at zero.

### Margin Requirements

**Initial collateral:**

$$ C_{A,\text{initial}} \geq \text{IM} \times Q \times K, \quad C_{B,\text{initial}} \geq \text{IM} \times Q \times K $$

where $\text{IM}$ = initial margin ratio (e.g. 15%).

**Mark-to-market at time $t$:**

$$ \text{UnrealizedP\&L}_{A}(t) = (S_t - K) \times Q $$

**Margin check:**

$$ C_A(t) - \max(S_t - K, 0) \cdot Q \geq m \cdot Q \cdot K $$

$$ C_B(t) - \max(K - S_t, 0) \cdot Q \geq m \cdot Q \cdot K $$

If violated → **margin call**: position enters grace period; if not remedied → **liquidation**.

### Numerical Example

```
Open:
  Producer A wants $40/REC floor for 5,000 RECs
  Offtaker B willing to cap at $40/REC for 5,000 RECs
  → Strike K = $40, Quantity Q = 5,000
  → IM = 15% → each posts $40 × 5,000 × 15% = $30,000 collateral
  → Settlement: 2026-12-31

Scenario 1: S_T = $55 (bullish)
  Net = ($55 − $40) × 5,000 = $75,000
  → A pays B $75,000
  → B's total return: +$75,000 (150% ROI on $30k collateral)
  → A effective REC price: $40 (capped) ✓

Scenario 2: S_T = $30 (bearish)
  Net = ($40 − $30) × 5,000 = $50,000
  → B pays A $50,000
  → A's effective REC price: $40 (floor) ✓
  → B effective REC price: $30 (saved $10/REC)

Scenario 3: S_T = $40 (at-the-money)
  Net = $0
  → No transfer; both get collateral back
```

---

## 💰 Tokenomics & Fee Model

### Protocol Fees

| Fee Type | Rate | Payer | Recipient |
|----------|------|-------|-----------|
| Spot trading fee | 0.10% (10 bps) | Both sides (taker) | Protocol treasury |
| CfD opening fee | 0.05% (5 bps) of notional | Both parties | Protocol treasury |
| CfD settlement fee | $5 flat per position | Party paying difference | Protocol treasury |
| Oracle attestation fee | $0.01 per reading | Generator (deducted at mint) | Oracle node operators |
| REC retirement fee | $2 flat per batch | Retirer | Protocol treasury |
| Market maker rebate | −0.02% (−2 bps) | Protocol → maker | Maker side |

### Fee Distribution

```
Protocol Fees Collected
        │
        ▼
  ┌─────────────────────────────────┐
  │  50% → Protocol Treasury        │ ← governance-controlled spending
  │  30% → Liquidity Rewards        │ ← distributed to active LPs
  │  15% → Oracle Staking Pool      │ ← distributed to honest oracle nodes
  │   5% → Ecosystem Grants         │ ← builders, integrations, research
  └─────────────────────────────────┘
```

### Inflation & Supply Caps

- No protocol token inflation (`stellar-rec` uses yUSDC as quote currency)
- REC token supply = total MWh generated and verified (tied to real-world generation)
- Protocol sustainability via fee revenue, not token emissions

---

## 🗃 Data Model

### REC Token

```
REC {
    token_id: u64,             // auto-increment
    asset_id: Bytes(32),       // hash of asset metadata
    generation_timestamp: u64, // unix seconds
    fuel_type: u8,             // 0=solar, 1=wind, 2=hydro, 3=other
    vintage_year: u16,
    metadata_uri: Bytes(64),   // IPFS / ARWeave CID
    owner: Address,
    state: u8,                 // 0=active, 1=retired
    retired_at: Option<u64>,
    retirement_receipt: Option<Bytes(32)>
}
```

### Order

```
Order {
    order_id: u64,
    trader: Address,
    side: u8,              // 0=buy, 1=sell
    price: i128,           // in yUSDC (7 decimals)
    initial_qty: u64,      // RECs
    remaining_qty: u64,
    timestamp: u64,
    restrictions: u8,      // 0=none, 1=FOK, 2=IOC
    vintage_filter: Option<u16>,
    status: u8             // 0=open, 1=filled, 2=cancelled
}
```

### CfD Position

```
CfDPosition {
    position_id: u64,
    counterparty_a: Address,    // producer / long
    counterparty_b: Address,    // offtaker / short
    strike_price: i128,         // yUSDC per REC
    quantity: u64,              // RECs
    settlement_date: u64,
    collateral_a: i128,
    collateral_b: i128,
    maintenance_margin_bps: u16,
    oracle_feed_id: Bytes(8),
    state: u8,                  // 0=pending, 1=active, 2=settled, 3=expired, 4=liquidated
    last_mtm_timestamp: u64,
    mtm_value: i128              // latest mark-to-market P&L
}
```

### Retirement Receipt

```
RetirementReceipt {
    receipt_id: Bytes(32),
    retirer: Address,
    token_ids: Vec<u64>,
    total_mwh: u64,
    claim_period_start: u64,
    claim_period_end: u64,
    purpose: Bytes(128),
    jurisdiction: Bytes(32),
    merkle_root: Bytes(32),
    timestamp: u64,
    block_height: u64
}
```

---

## 📡 Event & Error Reference

### Events (prefix: contract-specific)

All events use a two-topic structure: `(contract_symbol, event_symbol)`. Event topics are `symbol_short!` encoded.

#### REC Token — prefix `("rec", ...)`

| Symbol | Data Payload | Description |
|--------|-------------|-------------|
| `"mint"` | `(token_id, generation_timestamp, 1u64)` | Token minted |
| `"xfer"` | `(token_id, from, to)` | Token transferred |
| `"burn"` | `(token_id, caller)` | Token burned |
| `"paus"` | `()` | Contract paused |
| `"resm"` | `()` | Contract resumed |

#### Oracle Handler — prefix `("orcl", ...)`

| Symbol | Data Payload | Description |
|--------|-------------|-------------|
| `"reg"` | `(pubkey, operator)` | Oracle registered |
| `"rev"` | `(pubkey,)` | Oracle revoked |
| `"bond"` | `(pubkey, stake, active)` | Bond deposited |
| `"unbd"` | `(pubkey, stake)` | Bond withdrawn |
| `"fund"` | `(amount, pool)` | Reward pool funded |
| `"prce"` | `(price,)` | Reference price set |
| `"rewd"` | `(pubkey, amount)` | Rewards claimed |
| `"read"` | `(reading_hash, meter_id, mwh, oracle_count, token_id)` | Meter reading validated; REC minted |
| `"disp"` | `(reading_hash,)` | Dispute raised |
| `"resd"` | `(reading_hash, outcome)` | Dispute resolved |
| `"paus"` | `()` | Contract paused |
| `"resm"` | `()` | Contract resumed |

#### Marketplace — prefix `("mkt", ...)`

| Symbol | Data Payload | Description |
|--------|-------------|-------------|
| `"plac"` | `(order_id, trader, side)` | Order placed |
| `"canc"` | `(order_id,)` | Order cancelled |
| `"matc"` | `(buy_id, sell_id, fill_qty, fill_price, fee)` | Orders matched |
| `"cfdp"` | `(position_id, counterparty_a, strike, qty, expiry)` | CfD opened (pending) |
| `"cfdo"` | `(position_id, cpty_a, cpty_b, strike, qty, expiry)` | CfD accepted (active) |
| `"cfda"` | `(position_id, caller, amount)` | Collateral added |
| `"cfdr"` | `(position_id, caller, amount)` | Collateral removed |
| `"cfds"` | `(position_id, spot_price, payoff)` | CfD settled |
| `"cfdl"` | `(position_id, spot_price, payoff)` | CfD liquidated |
| `"paus"` | `()` | Contract paused |
| `"resm"` | `()` | Contract resumed |

#### Retirement — prefix `("ret", ...)`

| Symbol | Data Payload | Description |
|--------|-------------|-------------|
| `"retr"` | `(receipt_id, retirer, token_count, total_mwh, period_start, period_end)` | RECs retired; receipt issued |
| `"vrfy"` | `(verifier, authorized)` | Verifier set or removed |
| `"paus"` | `()` | Contract paused |
| `"resm"` | `()` | Contract resumed |

### Error Codes (per-contract)

#### REC Token (`RecTokenError`)

| # | Name | Description |
|---|------|-------------|
| 1 | `Unauthorized` | Caller is not the owner or authorized operator |
| 2 | `RecAlreadyRetired` | Token is already burned/retired |
| 3 | `InsufficientBalance` | Caller does not own this token |
| 4 | `TokenNotFound` | Token ID does not exist |
| 5 | `DuplicateMint` | Token with same asset + timestamp already exists |
| 6 | `ContractPaused` | Operation not allowed while paused |

#### Oracle Handler (`OracleError`)

| # | Name | Description |
|---|------|-------------|
| 1 | `Unauthorized` | Caller lacks required role |
| 2 | `OracleAlreadyRegistered` | Public key already registered |
| 3 | `OracleNotFound` | Oracle not found |
| 4 | `ThresholdNotMet` | Not enough valid signatures |
| 5 | `InvalidSignature` | Signature doesn't match registered oracle key |
| 6 | `InvalidMeterReading` | Meter reading out of bounds |
| 7 | `MeterNotBound` | Meter ID not registered |
| 8 | `AlreadyResolved` | Dispute already resolved |
| 9 | `DisputeWindowExpired` | Challenge period passed |
| 10 | `ContractPaused` | Operation not allowed while paused |
| 11 | `BondTooLow` | Oracle bond below minimum requirement |
| 12 | `InsufficientBond` | Not enough bond to cover slash |
| 13 | `NoRewardsToClaim` | No accumulated rewards for oracle |
| 14 | `RewardPoolInsufficient` | Reward pool balance too low |

#### Marketplace (`MarketError`)

| # | Name | Description |
|---|------|-------------|
| 1 | `Unauthorized` | Caller lacks required role |
| 2 | `OrderNotFound` | Order ID doesn't exist |
| 3 | `OrderFilled` | Order already filled or cancelled |
| 4 | `PriceMismatch` | Buy price < sell price |
| 5 | `InsufficientBalance` | Not enough balance for trade |
| 6 | `FeeCapExceeded` | Fee rate above governance max |
| 7 | `VintageMismatch` | REC vintage doesn't match order filter |
| 8 | `InvalidQuantity` | Quantity must be > 0 |
| 9 | `InsufficientCollateral` | Collateral below initial margin |
| 10 | `CollateralBelowMaintenance` | Margin call triggered |
| 11 | `PositionNotFound` | CfD position doesn't exist |
| 12 | `PositionNotActive` | Position not in active state |
| 13 | `CfDAlreadySettled` | Position already closed |
| 14 | `ContractPaused` | Operation not allowed while paused |

#### Retirement (`RetirementError`)

| # | Name | Description |
|---|------|-------------|
| 1 | `Unauthorized` | Caller lacks required role |
| 2 | `AlreadyRetired` | Token already retired |
| 3 | `TokenNotFound` | Token ID does not exist |
| 4 | `InvalidClaimData` | Claim period or data invalid |
| 5 | `ReceiptNotFound` | Receipt ID does not exist |
| 6 | `NoTokensProvided` | Token list is empty |
| 7 | `NotTokenOwner` | Caller does not own the specified token |
| 8 | `RecTokenNotSet` | REC token contract not configured |
| 9 | `DuplicateToken` | Same token in multiple positions of a single retire call |
| 10 | `ContractPaused` | Operation not allowed while paused |
| 11 | `VerifierAlreadySet` | Verifier already configured with same value |
| 12 | `VerifierNotFound` | Address is not a registered verifier |

---

## 🔀 Cross-Contract Interaction Flow

### Minting Flow (Detailed Sequence)

```
Meter            Oracle Node 1      Oracle Node 2      Oracle Handler    REC Token         Retirement
 │                     │                  │                   │                │                │
 │──reading(MWh,ts)──►│                  │                   │                │                │
 │                     │──verify(reading)│                   │                │                │
 │                     │──sign(hash)──┐  │                   │                │                │
 │                     │              │  │                   │                │                │
 │                     │              │  │──verify(reading)  │                │                │
 │                     │              │  │──sign(hash)──┐    │                │                │
 │                     │              │  │              │    │                │                │
 │                     │◄─────submit_reading(hash, sigs[2], proof)──────────►│                │
 │                     │              │                 │    │                │                │
 │                     │              │                 │    │──validate_sigs(3-of-5)          │
 │                     │              │                 │    │──plausibility_check(+)          │
 │                     │              │                 │    │                │                │
 │                     │              │                 │    │──cross_contract_call()         │
 │                     │              │                 │    │──────────mint()──────────────►  │
 │                     │              │                 │    │                │                │
 │                     │              │                 │    │                │──create_token()│
 │                     │              │                 │    │                │──assign_meta() │
 │                     │              │                 │    │                │                │
 │                     │              │                 │    │◄─────RecMinted(token_id)──────│
 │                     │              │                 │    │                │                │
 │                     │              │                 │    │──emit ValidatedReading()       │
 │                     │              │                 │    │                │                │
◄─────────────────────────────────────────────────────────────────────────────────────────────
```

### CfD Settlement Flow

```
Oracle Handler    Marketplace (CfD)      Party A (Producer)     Party B (Offtaker)
      │                   │                       │                      │
      │───spot_price──────►                       │                      │
      │                   │                       │                      │
      │                   │──settle_cfd(pos_id)──►│                      │
      │                   │◄──────consent─────────│                      │
      │                   │──settle_cfd(pos_id)──►│                      │
      │                   │◄──────consent─────────│                      │
      │                   │                       │                      │
      │                   │──calculate_S_T = $55  │                      │
      │                   │──net = (55-40)×5000   │                      │
      │                   │──net = $75,000        │                      │
      │                   │                       │                      │
      │                   │──transfer($75k)───────│─────────────────────►│
      │                   │──release_collateral──►│                      │
      │                   │──release_collateral──►│─────────────────────►│
      │                   │                       │                      │
      │                   │──emit CfDSettled(id)  │                      │
      │                   │                       │                      │
```

---

## 🏛 Governance Model

### Permission Model

| Role | Can | Assigned By |
|------|-----|-------------|
| **Admin** | Deploy contracts, transfer admin, pause/resume, set fees, upgrade | Initial deployer; transferable |
| **Oracle** | Submit readings, register meters | Admin |
| **Market Maker** | Match orders at preferred rate, access bulk matching | Admin |
| **Retirer** | Burn RECs, generate certificates | Any address (permissionless) |
| **Trader** | Place/cancel orders, open CfD positions | Any address |
| **Auditor** | Query any state, verify retirement | Any address (read-only) |

### Future Governance (Phase 6+)

- Transition to **DAO-controlled** parameters via Soroban token-based voting
- Parameters subject to governance:
  - Fee rates (within hard-coded bounds)
  - Oracle threshold
  - Maintenance margin ratios
  - Treasury allocation
  - Contract upgrades

---

## ⚖ Regulatory Compliance

### REC Standards Compliance

| Standard | Compliance | Notes |
|----------|-----------|-------|
| I-REC Standard | ✓ | Metadata schema aligns with I-REC EAC code |
| Green-e / APX | ✓ | Vintage, fuel type, asset granularity |
| RE100 Technical Criteria | ✓ | 12-month retirement matching window |
| GHG Protocol Scope 2 | ✓ | Market-based method supported |
| EECS (European Energy Certificate System) | ✓ | Guarantee of Origin (GO) compatible |
| California Air Resources Board (CARB) | TBD | Requires jurisdictional attestation |

### Anti-Fraud & Integrity

- **Meter tampering**: Multiple oracle attestations + plausibility bounds (±5% of expected output based on weather/capacity)
- **Double minting**: `asset_id + generation_timestamp` uniqueness constraint enforced by contract
- **Vintage fraud**: REC creation block timestamp compared to claimed generation time (rejects future-dated RECs)
- **Collusion**: N-of-M oracle model with slashing; economic disincentive for false reporting

### Jurisdictional Considerations

- Each REC carries `country` and `jurisdiction` fields
- Retirement contract validates that claim jurisdiction matches REC jurisdiction
- Bridge contracts for legacy registry RECs (I-REC → stellar-rec) include attestation from authorized registrar

---

## 🛠 Technical Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| **Blockchain** | Stellar (Soroban) | Low fees ($0.00001), fast finality (3–5s), carbon-friendly, built-in DEX primitive |
| **Smart Contracts** | Rust + Soroban SDK v22.x | Type-safe, WASM-compiled, deterministic execution |
| **Token Standard** | SEP-41 (Stellar Asset Contract) | Interoperable with Stellar ecosystem wallets/exchanges |
| **Oracle Network** | Custom Rust-based oracle with threshold BLS signatures | Decentralized, slashing-enabled |
| **Frontend** | React 19 + `@stellar/stellar-sdk` + Wagmi-ish bindings | Familiar developer experience |
| **Indexer** | Stellar RPC + custom sink to PostgreSQL | Real-time event stream for dashboards |
| **Testing** | `cargo test` · Soroban sandbox · `cargo expand` · fuzz testing | Full coverage including edge cases |
| **Formal Verification** | K Framework / SMT (planned) | Math-proven correctness for core mint/burn |
| **Deployment** | Soroban CLI · GitHub Actions · Docker | CI/CD pipeline with testnet → mainnet promotion |
| **Storage** | Soroban contract data (key-value) + IPFS for metadata | On-chain for state; off-chain for large blobs |

---

## 📊 Comparison With Existing Solutions

| Feature | I-REC Registry | APX / NREL | Voluntary Carbon Markets | **stellar-rec** |
|---------|---------------|-----------|-------------------------|-----------------|
| **Settlement time** | 30–90 days | 15–30 days | 7–45 days | **~5 seconds** |
| **Transaction cost** | $0.50–$2.00/REC | $0.25–$1.00/REC | 5–15% broker fee | **<$0.01/REC** |
| **Traceability** | Batch-level | Registry-level | Minimal | **Asset + hour granularity** |
| **Secondary market** | OTC / bilateral | Bilateral | Limited | **On-chain order book** |
| **Hedging instruments** | None | None | Forwards (illiquid) | **CfD smart contracts** |
| **Retirement proof** | PDF certificate | Portal receipt | Scanned document | **On-chain receipt + Merkle proof** |
| **Double-counting risk** | Moderate | Moderate | High | **Mathematically impossible** |
| **Global accessibility** | Limited (country-by-country) | US / NA only | Fragmented | **Permissionless, global** |
| **Interoperability** | Excel exports | API (paid) | None | **SEP-41, cross-chain bridges** |
| **Auditability** | Manual | Manual | Manual | **Real-time, public, programmable** |

---

## 🚀 Getting Started

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Soroban CLI
cargo install soroban-cli --locked

# Verify
rustc --version && cargo --version && soroban --version
```

### Clone & Build

```bash
git clone https://github.com/your-org/stellar-rec
cd stellar-rec

# Build all contracts (WASM)
cargo build --target wasm32-unknown-unknown --release

# Run all tests (unit + integration + sandbox)
cargo test

# Run with output
cargo test -- --nocapture
```

### Deploy to Testnet

```bash
# Generate keypair for testnet
soroban config identity generate rec-admin
soroban config identity address rec-admin

# Deploy REC Token contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/rec_token.wasm \
  --network testnet \
  --source rec-admin

# Initialize
soroban contract invoke \
  --id <REC_TOKEN_ID> \
  --network testnet \
  --source rec-admin \
  -- \
  initialize \
  --admin <ADMIN_PK>

# Verify
soroban contract invoke \
  --id <REC_TOKEN_ID> \
  --network testnet \
  -- \
  total_supply

# Deploy remaining contracts (Oracle, Marketplace, Retirement)
# and wire them with cross-contract IDs
```

### Local Development Sandbox

```bash
# Start Soroban dev server
soroban network start

# Deploy to local sandbox
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/rec_token.wasm \
  --network local

# Run integration test suite
cargo test --test integration
```

---

## 📖 API Reference

### REC Token Contract

| Function | Signature | Returns | Access |
|----------|-----------|---------|--------|
| `initialize` | `(admin: Address)` | `()` | Admin (once) |
| `mint` | `(to: Address, amount: u64, metadata_uri: Bytes)` | `u64 (token_id)` | Oracle Handler |
| `burn` | `(token_id: u64)` | `()` | Owner / Retirement |
| `transfer` | `(from: Address, to: Address, token_id: u64)` | `()` | Owner / Approved |
| `balance_of` | `(owner: Address)` | `u64` | Anyone |
| `token_uri` | `(token_id: u64)` | `Bytes` | Anyone |
| `total_supply` | `()` | `u64` | Anyone |
| `owner_of` | `(token_id: u64)` | `Address` | Anyone |
| `is_retired` | `(token_id: u64)` | `bool` | Anyone |

### Oracle Handler Contract

| Function | Signature | Returns | Access |
|----------|-----------|---------|--------|
| `initialize` | `(admin: Address, rec_token: Address)` | `()` | Admin (once) |
| `register_oracle` | `(pubkey: Bytes, uri: Bytes)` | `()` | Admin |
| `revoke_oracle` | `(pubkey: Bytes)` | `()` | Admin |
| `set_threshold` | `(numerator: u32, denominator: u32)` | `()` | Admin |
| `submit_reading` | `(asset_id: Bytes, mwh: u64, ts: u64, sigs: Vec<Signature>)` | `()` | Oracle |
| `set_meter` | `(meter_id: Bytes, asset_id: Bytes)` | `()` | Admin |
| `pause` | `()` | `()` | Admin |
| `resume` | `()` | `()` | Admin |
| `oracle_count` | `()` | `u32` | Anyone |

### Marketplace Contract

| Function | Signature | Returns | Access |
|----------|-----------|---------|--------|
| `initialize` | `(admin: Address, rec_token: Address, oracle: Address)` | `()` | Admin (once) |
| `place_order` | `(side: u8, price: i128, qty: u64, restrictions: u8)` | `u64 (order_id)` | Anyone |
| `cancel_order` | `(order_id: u64)` | `()` | Order owner |
| `match_orders` | `(buy_id: u64, sell_id: u64)` | `(fill_qty: u64, fill_price: i128)` | Market Maker / Cron |
| `open_cfd` | `(strike: i128, qty: u64, expiry: u64, collateral: i128)` | `u64 (position_id)` | Anyone |
| `accept_cfd` | `(position_id: u64, collateral: i128)` | `()` | Counterparty |
| `settle_cfd` | `(position_id: u64)` | `(net: i128)` | Either party |
| `add_collateral` | `(position_id: u64, amount: i128)` | `()` | Position party |
| `remove_collateral` | `(position_id: u64, amount: i128)` | `()` | Position party (excess only) |
| `liquidate` | `(position_id: u64)` | `()` | Admin / Cron |
| `set_fee_rate` | `(bps: u16)` | `()` | Admin (≤100 bps) |
| `get_order` | `(order_id: u64)` | `Order` | Anyone |
| `get_cfd` | `(position_id: u64)` | `CfDPosition` | Anyone |
| `get_best_bid` | `()` | `(price: i128, qty: u64)` | Anyone |
| `get_best_ask` | `()` | `(price: i128, qty: u64)` | Anyone |

### Retirement Contract

| Function | Signature | Returns | Access |
|----------|-----------|---------|--------|
| `initialize` | `(admin: Address, rec_token: Address)` | `()` | Admin (once) |
| `retire` | `(token_ids: Vec<u64>, claim_data: Bytes, purpose: Bytes, jurisdiction: Bytes)` | `Bytes (receipt_id)` | Token owner |
| `get_receipt` | `(receipt_id: Bytes)` | `RetirementReceipt` | Anyone |
| `verify_retirement` | `(token_id: u64)` | `(bool, Option<Bytes>)` | Anyone |
| `prove_claim` | `(claimer: Address, period_start: u64, period_end: u64)` | `Bytes (Merkle proof)` | Anyone |
| `set_verifier` | `(contract: Address, authorized: bool)` | `()` | Admin |

---

## 🔒 Security & Risk

### Security Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    SECURITY LAYERS                              │
│                                                                 │
│  1. Access Control   ─── Role-based (admin, oracle, etc.)       │
│  2. Pausability      ─── Emergency pause per contract           │
│  3. Safe Math        ─── Overflow-checked arithmetic            │
│  4. Reentrancy Guard ─── Mutex on cross-contract calls          │
│  5. Rate Limiting    ─── Max mint per block                     │
│  6. Collateralization ─── Over-collateralized CfD positions     │
│  7. Oracle Security  ─── N-of-M threshold + slashing            │
│  8. Audit Trail      ─── All operations emit events             │
│  9. Upgradeability   ─── Proxy pattern with timelock            │
│  10. Formal Verification  ── K Framework (planned)              │
└─────────────────────────────────────────────────────────────────┘
```

### Risk Factors

| Risk | Description | Mitigation |
|------|-------------|-----------|
| **Oracle compromise** | Malicious oracles submit false meter readings | N-of-M threshold; stake slashing; dispute windows |
| **Smart contract bugs** | Exploit in mint/market/CfD logic | Professional audit; bug bounty; formal verification |
| **Regulatory uncertainty** | REC recognition on-chain varies by jurisdiction | Modular compliance layer; jurisdiction-aware metadata |
| **Market manipulation** | Wash trading, spoofing on order book | Volume checks; market surveillance; circuit breakers |
| **Liquidity risk** | Thin order books lead to high slippage | Market maker incentives; CfD as alternative |
| **Collateral volatility** | yUSDC depeg | Only use audited stablecoins; emergency settlement |
| **Meter tampering** | Physical meter manipulated to report false output | Cross-reference with grid data; plausibility bounds |
| **Front-running** | Validators see CfD settlement oracle price before execution | Commit-reveal for oracle prices; batch settlement |

### Audits & Formal Verification

| Scope | Type | Status |
|-------|------|--------|
| REC Token (mint/burn path) | Formal verification (SMT) | Planned |
| CfD settlement math | Formal verification (K Framework) | Planned |
| Full contract suite | Third-party security audit | TBD (Phase 6) |
| Oracle signature verification | Cryptography review | TBD |
| Economic security (CfD margin) | Game-theoretic analysis | In progress |

---

## 🗺 Roadmap

| Phase | Timeline | Milestone | Deliverables |
|-------|----------|-----------|-------------|
| **0** | 🟢 Complete | **REC Token Contract** | SEP-41 token; metadata schema; mint/burn/test suite |
| **1** | 🟡 In progress | **Oracle Handler** | Oracle registration; threshold signing; meter ingestion; dispute mechanism |
| **2** | 🔜 Q3 2026 | **Spot Marketplace** | Order book; matching engine; fee model; frontend (basic) |
| **3** | 🔜 Q4 2026 | **CfD Engine** | Position management; margin engine; liquidation; settlement |
| **4** | 🔜 Q1 2027 | **Retirement Registry** | Burn + certificate; Merkle proofs; verifier portal |
| **5** | 🔜 Q2 2027 | **Frontend Suite** | Dashboard; explorer; wallet integration; analytics |
| **6** | 🔜 Q3 2027 | **Mainnet Launch** | Third-party audit; bug bounty; governance vote; mainnet deploy |
| **7** | 🔭 Future | **Cross-Chain Bridges** | I-REC legacy bridge; EVM interoperability; L2 settlement |

---

## ❓ FAQ

**Q: What is a REC (Renewable Energy Certificate)?**  
A: A REC is a market-based instrument that represents the environmental attributes of 1 MWh of renewable electricity generation. It's the currency of green energy claims.

**Q: Why Stellar / Soroban instead of Ethereum?**  
A: Stellar offers sub-cent transaction fees (~$0.00001), 5-second finality, built-in decentralized exchange primitives, and is itself carbon-neutral — aligning with our green mission. Soroban is Rust-based (safe, fast, formally verifiable).

**Q: What is a CfD and why does it matter for RECs?**  
A: A Contract-for-Difference is a derivative that lets producers lock in a minimum REC price and buyers lock in a maximum price. It enables hedging without transferring the underlying RECs until (if ever) physical delivery is needed. This is critical for PPA-backed renewable projects.

**Q: How do you prevent double counting?**  
A: Each REC is minted once and can only be retired (burned) once. The retirement is recorded permanently on-chain with a Merkle proof. An auditor can verify within seconds that a specific REC has been retired and cannot be reused.

**Q: Is this replacing I-REC / APX / Green-e?**  
A: Not replacing — complementing. We provide a settlement layer and marketplace for RECs that are also compatible with legacy registries. Bridge contracts allow I-REC-certified RECs to be represented on-chain with registrar attestation.

**Q: How much does it cost to mint a REC?**  
A: Network fee: ~$0.000002. Oracle attestation fee: ~$0.01. Total: <$0.02 per REC minted (vs $0.50–$2.00 via traditional registries).

**Q: What prevents someone from minting RECs for electricity that wasn't actually generated?**  
A: (1) Multiple independent oracle nodes cross-verify meter data. (2) Plausibility bounds check against expected output. (3) Oracle nodes post stake that can be slashed for fraudulent attestations. (4) Dispute window allows challenges.

**Q: Can I use this for RE100 / GHG Protocol reporting?**  
A: Yes. The retirement contract issues verifiable credentials compatible with RE100 reporting requirements. The Merkle proof system allows selective disclosure of your REC portfolio to auditors without revealing your entire position.

**Q: What wallets / DEXes are supported?**  
A: Any Stellar Soroban-compatible wallet (e.g. Freighter, Albedo). The SEP-41 standard ensures compatibility with Stellar ecosystem exchanges.

---

## 🤝 Contributing

We welcome contributions. Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines.

**Ways to contribute:**
- 🐛 Report bugs via GitHub Issues
- 💡 Suggest features / improvements
- 🔧 Submit PRs for open issues
- 🔬 Security research (bug bounty pending)
- 🌍 Translate documentation
- 📊 Run an oracle node (testnet → mainnet)

### Development Workflow

```bash
# Fork + clone
git clone https://github.com/your-org/stellar-rec
cd stellar-rec

# Create feature branch
git checkout -b feat/your-feature

# Make changes, write tests, ensure all pass
cargo test
cargo clippy -- -D warnings
cargo fmt --check

# Commit with conventional commit message
git commit -m "feat: add oracle slashing mechanism"

# Push and open PR
git push origin feat/your-feature
```

### Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation
- `test:` — test additions
- `refactor:` — code change without feature/fix
- `audit:` — security / audit improvement

---

## 📄 License

**MIT License** — see [LICENSE](./LICENSE) for details.

```
Copyright (c) 2026 stellar-rec contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.
```

---

<div align="center">

**Powering the green transition, one MWh at a time.** 🌱

[Stellar](https://stellar.org) · [Soroban](https://soroban.stellar.org) · [Rust](https://www.rust-lang.org)

<sub>stellar-rec · #DecarbonizeDeFi · #RECsOnChain</sub>

</div>

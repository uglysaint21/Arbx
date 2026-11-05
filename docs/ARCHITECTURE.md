# ArbX — Architecture Proposal

> **Status:** Draft v0.1 — proposed architecture, not yet approved.
>
> **Purpose:** Translate the agreed product behaviour into a practical software structure. This document proposes boundaries, responsibilities, data flow, security controls, and a staged build plan. Decisions that materially affect the design are marked **OPEN** and require confirmation before implementation.
>
> **Related document:** [Product Specification](./PRODUCT_SPECIFICATION.md)

---

## 1. Architecture goals

The architecture must support the product as currently defined:

- A locally operated Windows desktop application, ultimately distributed as an installer/executable.
- A graphical dashboard, strategy editor, settings, portfolio view, and trade history.
- Continuous monitoring of configured token pairs across multiple DEXs on the same chain.
- Quote comparison and net-profit evaluation before any trade is submitted.
- Local persistence of strategies, settings, trade records, and application state.
- Strong separation between the user interface, trading decisions, wallet signing, and external blockchain/DEX services.

The design should be understandable and testable. The first version should be a **modular monolith**, not a collection of microservices. The modules should have clear interfaces so that chain and DEX integrations can be added without rewriting the application.

### Non-goals for the first version

- Cross-chain arbitrage.
- Cloud-hosted execution or a mandatory ArbX account.
- A public API or multi-user server.
- Copy trading, social features, or a browser-based trading terminal.
- Automatic live trading before simulation, risk controls, and transaction lifecycle handling are proven.

---

## 2. Proposed technology direction

The following is a recommendation for discussion, not a locked decision.

| Area | Proposed choice | Reason |
|---|---|---|
| Desktop shell | **Tauri 2** | Produces native desktop bundles and separates the web UI from privileged native operations. |
| UI | **React + TypeScript** | Component-based dashboard and a mature ecosystem for forms, charts, and state management. |
| UI styling | CSS variables + a small component/design system | Makes the dark/orange/frosted-glass visual language consistent without coupling business logic to styling. |
| Native application core | **Rust** | Owns privileged operations, background tasks, persistence boundaries, and transaction-signing operations. |
| Local database | **SQLite** | Durable, local, single-user storage for strategies, trade lifecycle records, and app settings. |
| Database access | Rust SQLx (or an equivalent typed Rust SQLite layer) | Keeps database access in the native core rather than exposing the database directly to the UI. |
| EVM chain access | Rust EVM client stack, to be selected after chain scope is confirmed | Keeps RPC, quote, simulation, and transaction operations behind a chain adapter. |
| Secrets | Windows-protected credential storage, accessed only by the native core | Avoids storing raw private keys in ordinary application settings or the UI. |
| Testing | Unit tests, integration tests against local/forked chains, and UI tests | Allows trading logic to be tested without risking real funds. |
| Packaging | Tauri Windows installer/EXE output | Matches the intended Windows desktop delivery. |

**Why Tauri is the current recommendation:** ArbX needs a rich desktop UI but also handles sensitive wallet material. Tauri provides a web frontend with a Rust core and a permissioned message-passing boundary. Its Windows build uses WebView2 and the Windows C++ build toolchain. This is a good fit if we are comfortable maintaining both TypeScript and Rust.

**Alternative:** Electron + React + TypeScript would keep more of the application in one language and has a mature desktop ecosystem. It ships Chromium and Node.js, however, and requires strict process isolation, sandboxing, IPC validation, and careful handling of privileged operations. We should choose Electron if one-language development speed is more important than a smaller native shell and Rust boundary.

**OPEN:** Confirm Tauri or Electron before scaffolding the application.

---

## 3. High-level system shape

The application is one local desktop product with these logical areas:

1. **Presentation layer** — screens, forms, charts, notifications, and user interaction.
2. **Application layer** — coordinates user actions and background workflows.
3. **Domain layer** — strategy rules, opportunity evaluation, profitability calculations, risk checks, and trade state transitions.
4. **Integration layer** — chain RPC, DEX quoting/routing, transaction simulation/submission, and market data.
5. **Security layer** — authentication, secret access, signing, and privileged command validation.
6. **Persistence layer** — local database and non-secret application preferences.

The UI must not call DEX contracts, sign transactions, or read private keys directly. It sends typed requests to the application core. The core validates the request and coordinates the relevant domain and integration services.

### Logical component map

```text
┌─────────────────────────────────────────────────────────────┐
│                    ArbX Desktop Application                 │
│                                                             │
│  ┌──────────────────────┐       ┌────────────────────────┐  │
│  │ Presentation (React) │◄─────►│ Typed App Command API  │  │
│  │ Dashboard / History  │       │ Tauri IPC boundary     │  │
│  │ Strategies / Settings│       └───────────┬────────────┘  │
│  └──────────────────────┘                   │               │
│                                             ▼               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                  Application Core (Rust)              │  │
│  │                                                       │  │
│  │  Auth & Session   Strategy Service   Bot Supervisor  │  │
│  │  Portfolio        History Service    Settings         │  │
│  │                                                       │  │
│  │  ┌──────────────── Domain / Trading Engine ────────┐  │  │
│  │  │ Monitor → Quote → Evaluate → Risk → Simulate    │  │  │
│  │  │                         → Approve → Execute      │  │  │
│  │  └───────────────────────────┬──────────────────────┘  │  │
│  └──────────────────────────────┼────────────────────────┘  │
│                                 │                           │
│              ┌──────────────────┼──────────────────┐        │
│              ▼                  ▼                  ▼        │
│        Chain adapters     DEX adapters       Secure signer  │
│        RPC / balances     Quotes / routes    Secret vault   │
│        receipts / logs    simulation         OS protection  │
│              │                  │                  │        │
│              └──────────────────┼──────────────────┘        │
│                                 ▼                           │
│                     SQLite + app preferences               │
└─────────────────────────────────────────────────────────────┘
              │
              ▼
       External RPC nodes, DEX contracts,
       and blockchain networks
```

This is a logical map, not a final crate/module or folder layout.

---

## 4. Component responsibilities

### 4.1 Presentation layer

Responsible for:
- Login and first-run password setup.
- Dashboard, portfolio/P&L, strategy list, strategy editor, Settings, About, and History.
- Displaying bot state and strategy monitoring status.
- Displaying quote freshness, estimated costs, trade status, errors, and notifications.
- Collecting user inputs and presenting validation errors.

It must not:
- Hold private keys or decrypted wallet secrets.
- Decide whether an opportunity is profitable.
- Construct and submit raw transactions.
- Be the source of truth for bot state or trade status.

### 4.2 Application command boundary

Expose a small, typed set of commands/events between the UI and native core. Examples of command concepts:

- Authenticate / lock application.
- Create, update, list, and delete a strategy.
- Start and stop the bot.
- Read dashboard and portfolio snapshots.
- Read paginated trade history.
- Add, replace, or remove a wallet credential.
- Read application settings.
- Subscribe to bot, quote, and trade lifecycle events.

Every command must validate its input in the native core. A hidden UI control is not a security boundary.

### 4.3 Authentication and session service

Responsibilities:
- First-run password creation and subsequent unlock.
- Password verification using a modern password-hashing function with a unique salt.
- Session lock and inactivity/explicit lock behaviour.
- Re-authentication before sensitive wallet changes or other high-impact actions, if adopted.

The application password is not the wallet private key and must never be used as a transaction-signing key.

**OPEN:** Password recovery and session timeout policy need product decisions.

### 4.4 Strategy service

Responsibilities:
- Validate strategy inputs against supported chains, tokens, and DEX deployments.
- Persist strategy configuration.
- Enable/disable individual strategies.
- Keep strategy identity stable so historical trades remain associated with a deleted strategy.
- Reject unsupported combinations rather than silently substituting a chain or DEX.

A strategy should refer to chain and token identifiers, not just display symbols. Token symbols are not unique.

### 4.5 Bot supervisor and monitoring service

Responsibilities:
- Own the authoritative ON/OFF state.
- Start and stop strategy monitoring tasks.
- Prevent duplicate workers for the same strategy.
- Apply per-chain request limits and backoff.
- Detect stale or unavailable RPC/DEX data.
- Publish status and error events to the UI.
- Recover safely after a restart.

Initial safety default proposed: **the bot starts OFF after every application restart**. The user must explicitly start it again. The application should persist whether it was previously running for diagnostics, but should not automatically resume live trading.

**OPEN:** Confirm restart behaviour and whether the bot can ever be configured to auto-start.

### 4.6 Market data and quote service

Responsibilities:
- Request comparable quotes for the same token pair and trade size from each configured DEX.
- Normalise quote results into a common internal format.
- Include quote timestamp/block reference, input amount, expected output, price impact, and available fee data.
- Reject expired, incomplete, or incomparable quotes.
- Track RPC health and quote latency.

Quotes must be compared for the same input amount and direction. Comparing headline token prices without considering trade size and price impact is not sufficient.

### 4.7 Opportunity and profitability engine

Responsibilities:
- Generate candidate routes from the configured DEX venues.
- Calculate expected output for the complete proposed route.
- Estimate network gas, DEX fees, token transfer taxes where detectable, and any other configured execution costs.
- Convert costs and expected profit into a common valuation unit for comparison.
- Apply the user's minimum net-profit and risk thresholds.
- Return a structured decision with reasons, not merely a boolean.

The engine should use integer/decimal-safe arithmetic for token amounts. Floating-point arithmetic must not be used for on-chain token quantities or financial calculations where precision loss could change a decision.

An opportunity is not executable simply because one DEX displays a lower spot price than another.

### 4.8 Risk and execution service

Responsibilities:
- Enforce maximum trade size, minimum net profit, slippage bounds, quote age, gas limits, and other configured risk constraints.
- Simulate the exact transaction or route where the chain/DEX supports it.
- Validate chain ID, token addresses, router/spender addresses, transaction value, deadline, nonce, and calldata.
- Request a signature from the secure signer only after validation.
- Submit the transaction and track its lifecycle through confirmation or failure.
- Reconcile actual token balances and realised outcomes after execution.

A transaction that has been submitted is not the same as a transaction that has been confirmed. The UI and History must distinguish pending, confirmed, reverted, and unknown outcomes.

**Safety rule:** no live transaction submission in the first implementation milestone. Build quote evaluation and simulation first. Enable live execution only after test coverage and explicit risk policy are agreed.

### 4.9 Chain adapters

A chain adapter provides a common interface for chain-specific operations, such as:
- Chain identity and supported-token validation.
- RPC health and block reference.
- Native/token balance reads.
- Gas estimation.
- Transaction simulation and submission.
- Receipt and confirmation tracking.

Chain-specific logic stays inside its adapter. The domain engine should not contain scattered chain-specific conditionals.

**OPEN:** Whether the first release supports EVM chains only or also Solana. Solana is a materially different transaction and DEX integration model and should not be assumed to fit an EVM adapter.

### 4.10 DEX adapters

Each DEX adapter encapsulates the exchange-specific details:
- Supported chain deployments.
- Factory/router/quoter or equivalent contract addresses.
- Quote acquisition.
- Route construction.
- Fee-tier and pool selection where relevant.
- Simulation compatibility.
- Transaction encoding requirements.

A DEX is enabled only for chain deployments that have been explicitly configured and verified. The product must not assume that Uniswap, PancakeSwap, or SushiSwap exists on every supported network.

### 4.11 Wallet and signing service

Responsibilities:
- Store wallet secrets in a protected local secret store.
- Decrypt or access a secret only inside the native core when signing is required.
- Never return private key material to the UI or persist it in SQLite, logs, crash reports, or analytics.
- Validate the intended transaction before signing.
- Support an explicit wallet-address display and ownership verification flow.

**OPEN:** Choose between imported private key, external wallet connection, or both. A MetaMask browser extension should not be assumed to be directly available to a standalone desktop WebView.

### 4.12 Persistence service

SQLite stores durable application records, including:
- Strategies and their configuration.
- Non-secret application settings.
- Trade and transaction lifecycle records.
- Historical quote/decision snapshots needed to explain why a trade was or was not taken.
- Portfolio snapshots or valuation points used for the P&L graph.
- Schema migration version.

Private keys, seed phrases, passwords, and decrypted secrets do not belong in SQLite.

---

## 5. Core trading workflow

The following is the proposed control flow. It intentionally separates opportunity detection from transaction execution.

```text
User starts bot
      │
      ▼
Bot supervisor loads enabled strategies
      │
      ▼
For each strategy, monitor supported DEX venues
      │
      ▼
Fetch comparable quotes for configured trade size
      │
      ▼
Are quotes complete, fresh, and comparable?
      ├── No → record diagnostic status; retry with backoff
      │
      ▼ Yes
Build candidate route(s)
      │
      ▼
Estimate final output and all known costs
      │
      ▼
Does net profit pass configured threshold?
      ├── No → store opportunity decision (not a completed trade)
      │
      ▼ Yes
Run risk checks and transaction simulation
      │
      ▼
Does simulation and every risk check pass?
      ├── No → reject; record reason
      │
      ▼ Yes
[Live execution remains disabled until approved]
      │
      ▼
Construct and validate exact transaction
      │
      ▼
Secure signer authorises transaction
      │
      ▼
Submit transaction and track receipt
      │
      ▼
Reconcile actual output, fees, and balances
      │
      ▼
Persist realised outcome and update dashboard
```

### Trade lifecycle states

The internal lifecycle should distinguish at least:

- `Detected` — candidate opportunity identified.
- `Rejected` — profitability, freshness, risk, or simulation check failed.
- `ReadyForExecution` — checks passed; not yet submitted.
- `Submitted` — transaction sent to the network.
- `Pending` — waiting for chain confirmation.
- `Confirmed` — successful receipt observed.
- `Reverted` — transaction included but execution failed.
- `Failed` — failed before successful submission.
- `Unknown` — submission outcome cannot yet be established; reconciliation required.

Only confirmed and reconciled trades should be treated as completed realised trades in History.

### Turning the bot OFF

Proposed behaviour:
- Stop scheduling new quote and opportunity cycles.
- Cancel work that has not reached transaction submission.
- Do not pretend a submitted transaction can be cancelled by switching the bot off.
- Continue tracking and reconciling transactions already submitted.
- Display an explicit status if a trade is still pending.

This needs product confirmation, especially for the meaning of OFF while a transaction is pending.

---

## 6. Data model — conceptual

This is a conceptual model, not a final SQL schema.

| Record | Purpose | Important relationships |
|---|---|---|
| `Chain` | Supported network metadata | Referenced by strategies and transactions |
| `Token` | Chain-specific token identity and decimals | Belongs to a chain |
| `DexDeployment` | A DEX deployment on a particular chain | References a chain |
| `Strategy` | User-configured pair, venues, size, and thresholds | References chain, two tokens, and multiple DEX deployments |
| `Opportunity` | A quote comparison and decision snapshot | References a strategy |
| `Trade` | One attempted execution lifecycle | References strategy and opportunity |
| `Transaction` | One on-chain transaction and receipt | Belongs to a trade |
| `PortfolioSnapshot` | Asset/valuation snapshot for dashboard history | Timestamped |
| `AppSettings` | Non-secret application preferences | Local application scope |

Important constraints:
- Store token addresses in canonical form and retain the chain ID.
- Store token quantities as exact integer base units or exact decimal strings, never binary floating-point.
- Preserve trade records when a strategy is deleted. Use a snapshot of relevant strategy labels/configuration for historical display.
- Record both expected and realised values, including the valuation timestamp and unit.
- Keep secrets outside the database.

---

## 7. Security architecture

Security is a core design constraint because ArbX can eventually control wallet funds.

### Required controls

1. **UI/native boundary:** the frontend receives only the commands and data it needs. It never receives a private key or seed phrase after submission.
2. **Secret storage:** use a Windows-protected secret-storage mechanism, such as DPAPI-backed protection or a vetted OS credential store integration. Do not invent custom cryptography.
3. **Password storage:** store only a salted password hash using a modern password-hashing algorithm. The app password is not itself an encryption key unless a deliberate, reviewed key-derivation design is adopted.
4. **Least privilege:** expose only required Tauri capabilities and native commands.
5. **No secret leakage:** redact secrets from logs, errors, crash reports, telemetry, and debug output.
6. **Transaction validation:** validate chain, recipient/router, token, amount, allowance, calldata, slippage, deadline, and value before signing.
7. **Safe defaults:** bot OFF at startup; no live trading in early milestones; strict limits when live trading is eventually enabled.
8. **Dependency hygiene:** pin and audit dependencies; keep desktop runtime and cryptographic libraries updated.
9. **Database protection:** keep the local database in the app's user-data directory and protect sensitive data at the field/secret-store level.
10. **Recovery and backup:** define how users recover or intentionally reset access without exposing wallet secrets.

### Important limitation

Local encryption reduces exposure but does not make a compromised computer safe. Malware running as the logged-in user may still be able to interfere with the application or observe user activity. ArbX must not claim that local encryption makes funds risk-free.

---

## 8. Reliability and operational behaviour

- RPC timeouts, rate limits, stale quotes, and DEX errors must be visible and must fail closed for trading decisions.
- Retry transient read operations with bounded exponential backoff and jitter.
- Do not blindly retry transaction submission when the previous submission result is uncertain; reconcile by transaction hash/nonce first.
- Keep the UI responsive while monitoring and quote evaluation run in background tasks.
- Persist important state transitions before reporting success to the UI.
- On restart, recover pending transaction tracking and database state, but leave the bot OFF unless the user explicitly starts it.
- Use structured logs with correlation IDs for strategy, opportunity, trade, and transaction events. Never log secrets.
- Use schema migrations and maintain a recoverable local database backup strategy.

---

## 9. Testing strategy

Testing must be built alongside the trading engine, not added after it.

### Unit tests
- Token amount and decimal conversion.
- Quote normalisation.
- Net-profit calculation.
- Gas/cost conversion.
- Slippage and threshold rules.
- Strategy validation.
- Trade state transitions.
- Risk-policy enforcement.

### Integration tests
- Chain adapter against a local development chain or fork.
- DEX quote and route adapters against known fixtures and forked contracts.
- Simulation and transaction construction.
- SQLite persistence and migrations.
- Secret storage and lock/unlock flows.

### End-to-end tests
- First launch and password setup.
- Create, edit, enable, and delete strategy.
- Bot start/stop.
- History and portfolio updates.
- Restart while a transaction is pending.
- Error states: RPC outage, stale quote, rejected simulation, reverted transaction.

### Trading safety gates
- Deterministic replay of captured quote data.
- Paper-trading mode with no signing or broadcast.
- Forked-network tests with controlled balances.
- Explicit live-trading approval gate after risk and execution reviews.

---

## 10. Proposed build sequence

Build in small verifiable increments. Do not begin with live trading.

| Phase | Deliverable | Exit condition |
|---|---|---|
| 0. Architecture decisions | Confirm desktop framework, launch chains, wallet model, and first execution model | Open questions below are answered and architecture is approved |
| 1. Desktop foundation | Tauri shell, React UI, navigation, theme, app lifecycle | Windows development build opens reliably |
| 2. Local app security and persistence | Password flow, protected secret-store integration, SQLite migrations | Secrets never enter UI/database/logs; restart works |
| 3. Product UI foundation | Dashboard, strategy CRUD, settings, history with fixture data | User flows work without blockchain access |
| 4. Chain read-only integration | Chain registry, RPC health, token balances, token metadata | Read-only data works on the first approved chain |
| 5. DEX quote engine | First DEX adapter, then second adapter, comparable quotes | Repeatable quotes and stale-data rejection |
| 6. Opportunity evaluation | Route comparison, cost model, net-profit and risk checks | Unit/integration tests prove deterministic decisions |
| 7. Paper trading | Full trade lifecycle without signing/broadcast | Paper trades reconcile and appear correctly in History |
| 8. Transaction simulation | Exact route simulation and transaction validation | Simulation failure blocks execution reliably |
| 9. Live execution | Secure signing, submission, receipt tracking, limits | Only after explicit approval and a dedicated security review |
| 10. Release hardening | Windows installer, updates, logs, backups, recovery, docs | Reproducible release build and release checklist pass |

---

## 11. Decisions required before implementation

Please answer these first. They are the decisions that most strongly affect the architecture.

1. **Desktop framework:** Are you happy to use **Tauri 2 + React + TypeScript + Rust** (recommended), or would you prefer **Electron + React + TypeScript** to keep most of the code in JavaScript/TypeScript?
2. **Launch chains:** Should version 1 support **EVM chains only** (for example Polygon, Ethereum, BNB Chain, Base), or must it support **Solana from the beginning** too? Solana would require its own chain and DEX adapter family.
3. **Wallet model:** Should version 1 let users **import a private key**, connect an external wallet where technically supported, or support both? The original product brief mentions MetaMask/private-key details but does not settle the interaction.
4. **Initial trading model:** Is the intended trade a **two-leg same-chain arbitrage** (buy token A with token B on DEX 1, then sell token A for token B on DEX 2), with both legs executed as one atomic transaction/route where possible? Or are sequential independent transactions acceptable? This is critical to execution risk.
5. **First milestone:** Should we build a polished desktop shell with fixture/demo data first, or prioritise a read-only live connection to one chain and two DEXs?

### Proposed defaults if you want me to decide

- Tauri 2 + React + TypeScript + Rust.
- EVM-only for the first release; add Solana later through a separate adapter family if still required.
- Start with imported private key protected by Windows credential protection; add external wallet support later.
- Treat arbitrage as one atomic route/transaction where the protocol supports it. Do not assume two separately submitted swaps are risk-free.
- Build the desktop shell and local strategy/history flows first, then read-only chain integration, quotes, paper trading, simulation, and only then live execution.

---

## 12. Reference material

The proposed technology direction was checked against current official documentation:

- [Tauri architecture](https://v2.tauri.app/concept/architecture/) — Rust core, webview frontend, and message-passing integration.
- [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/) — Windows build requirements including C++ Build Tools and WebView2.
- [Tauri security capabilities](https://v2.tauri.app/security/capabilities/) — permission scoping between frontend windows and native capabilities.
- [Tauri Windows distribution](https://v2.tauri.app/distribute/windows-installer/) — Windows installer and setup executable targets.
- [Viem documentation](https://viem.sh/docs/getting-started) — TypeScript Ethereum client alternative if we choose a TypeScript-heavy integration layer.
- [Electron process model](https://www.electronjs.org/docs/latest/tutorial/process-model) and [security guide](https://www.electronjs.org/docs/latest/tutorial/security/) — alternative desktop framework and its isolation requirements.

These references support the framework comparison; they do not replace a later dependency and security review.

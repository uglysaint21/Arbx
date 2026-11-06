# ArbX Roadmap

This roadmap is a staged plan, not a claim that future capabilities already exist. The order deliberately places verification and read-only market data ahead of any transaction-signing or live-trading work.

## Status key

- **Implemented** — code and product surface exist in the repository.
- **Awaiting verification** — implementation exists, but the required build, integration, or owner acceptance test has not been confirmed.
- **Planned** — not yet implemented.
- **Decision required** — product or execution details must be settled before implementation can be considered complete.

## Phase 0 — Product definition and boundaries

**Status: substantially documented**

- [x] Clarify the product behaviour, user journeys, dashboard, settings, strategy model, history, trading intent, and visual direction.
- [x] Separate product specification from architecture documentation.
- [x] Set Polygon PoS (chain ID 137) as the version 1 chain scope.
- [x] Define same-chain, two-venue, two-leg arbitrage intent.
- [x] Record unresolved product and execution decisions rather than silently assuming answers.

Key documents: `docs/PRODUCT_SPECIFICATION.md`, `docs/ARCHITECTURE.md`.

## Phase 1 — Desktop foundation and local security

**Status: implemented; native Windows acceptance still matters**

- [x] Tauri 2 desktop shell with React, TypeScript, and Rust.
- [x] Local application password setup/login using Argon2id password hashing.
- [x] Manual lock and 15-minute inactivity lock.
- [x] SQLite local persistence and schema migration tracking.
- [x] Strategy create/list/delete persistence and native input validation.
- [x] Windows Credential Manager wallet import/removal boundary.
- [ ] Owner-run acceptance test of wallet import/removal on the target Windows machine.
- [ ] Decide and implement password recovery/reset behaviour.
- [ ] Decide whether session timeout is configurable.
- [ ] Database backup/restore UX.

## Phase 2 — Polygon connectivity and wallet reads

**Status: implementation present; verification pending**

- [x] Securely configure a Polygon PoS JSON-RPC endpoint.
- [x] Verify chain ID 137 and read latest block number.
- [x] Read POL balance.
- [x] Read selected ERC-20 symbol, decimals, and wallet balance.
- [x] Validate token contract address and contract bytecode.
- [x] Format token balances with integer arithmetic.
- [ ] Confirm a successful Windows frontend build and Rust test run.
- [ ] Run the application on Windows and verify RPC connectivity using a user-controlled endpoint.
- [ ] Verify POL balance and a known ERC-20 balance against an independent trusted wallet/provider view.
- [ ] Test invalid token addresses, non-contract addresses, RPC errors, timeout behaviour, and locked-session behaviour.
- [ ] Record test evidence without recording private keys or credential-bearing RPC URLs.

Do not mark this phase complete until the build and owner-run integration checks pass.

## Phase 3 — DEX connectivity and market data

**Status: planned**

- [ ] Resolve which DEX deployments on Polygon are actually supported and identify their router/factory or quote interfaces.
- [ ] Implement isolated read-only adapters for each selected DEX.
- [ ] Retrieve pool/route information and token-pair quotes.
- [ ] Normalize quote units, token decimals, fees, and route metadata.
- [ ] Handle stale data, RPC/provider failures, liquidity limits, and unsupported pools.
- [ ] Display live quote data with source, timestamp, and freshness status.
- [ ] Add deterministic mocked tests and Polygon fork/local integration tests.

No DEX should be presented as supported merely because its name appears in the original product brief.

## Phase 4 — Opportunity detection and profitability

**Status: planned; product decisions required**

- [ ] Compare the same token pair across at least two supported venues.
- [ ] Calculate the complete proposed round-trip output.
- [ ] Account for DEX fees, network gas, price impact, slippage, and other execution costs that apply.
- [ ] Define minimum net profit, safety margin, maximum trade size, and stale-quote rejection rules.
- [ ] Detect and suppress false opportunities caused by stale or incomparable quotes.
- [ ] Show the user the quote sources, assumptions, estimated costs, and net result.
- [ ] Add reproducible tests for profitable, unprofitable, borderline, and invalid opportunities.

**Decision required:** settle profit denomination, full cost model, minimum net profit, safety margin, funding token/source, and risk limits before this phase is treated as complete.

## Phase 5 — Atomic route design and simulation

**Status: planned; security-critical gate**

- [ ] Select an execution path that can perform both swap legs atomically in one transaction on Polygon.
- [ ] Prove the route's token flow and failure/revert behaviour.
- [ ] Simulate the exact proposed transaction against a fork or supported simulation environment.
- [ ] Verify minimum output, deadlines, slippage bounds, gas assumptions, and balance/allowance requirements.
- [ ] Prevent execution if quotes are stale, simulation fails, or risk limits are exceeded.
- [ ] Review the transaction lifecycle, reorg handling, and recovery states.
- [ ] Complete a security review before introducing signing.

Two separate swap submissions are not an acceptable implementation of the product's atomic arbitrage requirement.

## Phase 6 — Paper trading and operational controls

**Status: planned**

- [ ] Run the opportunity engine without signing or broadcasting transactions.
- [ ] Record hypothetical entry/exit, quote snapshots, estimated costs, and hypothetical outcomes.
- [ ] Compare simulated/paper outcomes with subsequent market data.
- [ ] Add bot start/stop semantics, safe shutdown, and in-progress operation handling.
- [ ] Add operational limits, alerts, and clear stale-data/error states.
- [ ] Validate that dashboard and history figures are derived from recorded data rather than placeholders.

## Phase 7 — Signing and testnet execution

**Status: planned; requires explicit security review**

- [ ] Design a dedicated native signing boundary; private keys must never enter React, logs, SQLite, or command arguments.
- [ ] Add explicit user approval and transaction preview.
- [ ] Test on a suitable test environment before any mainnet execution.
- [ ] Validate nonce management, gas estimation, chain ID, transaction hashes, confirmations, reverts, and replacement/cancellation handling.
- [ ] Add strict spend limits and fail-closed behaviour.
- [ ] Conduct code and security review before proceeding.

## Phase 8 — Controlled mainnet release

**Status: planned; do not begin before prior gates pass**

- [ ] Complete security review and end-to-end testing.
- [ ] Start with user-approved, very small trade limits.
- [ ] Require explicit configuration and clear risk acknowledgement.
- [ ] Monitor transaction outcomes and reconcile wallet balances/history.
- [ ] Provide a reliable emergency stop that prevents new work without corrupting in-flight transaction tracking.
- [ ] Prepare Windows packaging, signing, update, backup, and recovery procedures.

## Phase 9 — Dashboard, history, and release hardening

**Status: partially present as UI; live data integration planned**

- [ ] Replace illustrative portfolio and P&L figures with verified wallet/portfolio data.
- [ ] Add actual strategy monitoring state and live opportunity details.
- [ ] Persist completed, failed, reverted, and rejected execution states distinctly.
- [ ] Preserve trade history when a strategy is deleted.
- [ ] Add history filtering/export only after the underlying data model is stable.
- [ ] Complete accessibility, usability, performance, and Windows installer testing.

## Open decisions that block later phases

The product specification remains the source of truth for unresolved product choices. Highest priority decisions are:

1. Exact DEX deployments and supported Polygon execution routes.
2. Funding model and starting asset.
3. Profit denomination and complete cost model.
4. Minimum net profit, safety margin, maximum trade size, and other risk limits.
5. Bot start/stop behaviour and treatment of in-progress transactions.
6. Portfolio valuation currency and precise history fields.
7. Password recovery and strategy deletion confirmation.

## Immediate next checkpoint

The next work should focus on **Phase 2 verification**, not on enabling transaction execution:

1. Confirm the current PR and CI state.
2. Run the Windows frontend build and Rust tests.
3. Test the configured Polygon RPC connection.
4. Verify a POL balance and one known ERC-20 balance.
5. Record the result and fix any failures before beginning DEX adapters.

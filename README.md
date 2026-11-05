# ArbX

**ArbX** is a locally operated desktop application in early development for monitoring and executing token-pair arbitrage opportunities across decentralised exchanges (DEXs) on the same blockchain.

The intended application will let a user configure strategies, monitor prices across selected DEX venues, evaluate opportunities after transaction costs, and review portfolio performance and completed trades through a graphical dashboard. The long-term target is a standalone Windows application distributed as an `.exe`.

> **Project status:** The desktop shell, local password/session security, SQLite strategy persistence, Windows Credential Manager wallet storage, and user-configured read-only Polygon RPC health checks are implemented. DEX quotes, balances, transaction simulation, signing, and trading are not implemented.

## Product scope

The current product direction includes:

- A locally running, password-protected desktop application.
- Local protection and storage of wallet credentials.
- User-configured token-pair strategies.
- Same-chain arbitrage comparisons across supported DEXs.
- A profitability check that accounts for transaction fees before a trade is executed.
- A dashboard for portfolio balance, P&L, and active strategies.
- A trade history showing expected and realised outcomes.
- A dark, professional visual identity with typewriter-inspired typography, orange and white accents, frosted-glass panels, and restrained animations.

The initial DEX examples are Uniswap, PancakeSwap, and SushiSwap. The exact supported blockchains, DEX availability per chain, trading route, funding model, and several execution safeguards remain to be decided.

## Product documentation

The product behaviour and visual direction are documented in:

- **[ArbX Product Specification](docs/PRODUCT_SPECIFICATION.md)** — product overview, user flows, dashboard, settings, history, arbitrage behaviour, visual direction, boundaries, and open product decisions.
- **[ArbX Architecture](docs/ARCHITECTURE.md)** — confirmed stack and chain scope, application boundaries, atomic trading workflow, security model, data concepts, testing strategy, and build phases.
- **[Development Guide](docs/DEVELOPMENT.md)** — prerequisites and commands for running the current preview.

The product specification intentionally does not define the software architecture or implementation. Those will be documented separately after the remaining product decisions are clarified.

## Current development stage

The Tauri + React + TypeScript + Rust desktop shell is in the repository. Local authentication, SQLite strategy persistence, secure wallet credential storage, and a read-only Polygon PoS RPC endpoint/health-check flow are implemented. Dashboard portfolio figures and chart remain illustrative; the strategy configuration is persisted locally, but no DEX data or token balances are queried yet. Live trading is deliberately deferred until the atomic route, simulation, risk controls, and transaction lifecycle have been implemented and reviewed.

Until then, this repository is the project’s source of truth for agreed product documentation. Work should be committed to the `main` branch unless the project workflow is deliberately changed.

## Scope and safety notes

ArbX is intended to interact with cryptocurrency wallets and execute on-chain transactions. Any eventual implementation must treat wallet credentials and transaction execution as security-critical. The product specification records the intended user experience; it is not a security audit, trading guarantee, or implementation claim.

## Repository

Source code and project documentation: [github.com/uglysaint21/Arbx](https://github.com/uglysaint21/Arbx)


## Current implementation status

The desktop shell now includes first-launch password setup, login, manual locking, 15-minute inactivity locking, local SQLite strategy persistence, and a Windows-native wallet import/removal flow. Imported private keys are stored in the current Windows user's Credential Manager; only the derived public address is shown in the UI and saved in SQLite.

The wallet import flow has not been tested with a real user key. That test is intentionally deferred to the owner's final acceptance step. ArbX still has no Polygon RPC integration, DEX quote engine, paper trading, transaction simulation, signing, or live trading. Dashboard figures remain illustrative.


## Polygon read-only connectivity

The native Settings page now supports a user-supplied Polygon PoS JSON-RPC endpoint. The endpoint is entered through a native Windows credential prompt and stored in Windows Credential Manager. ArbX validates the provider's chain ID (137) and reads the latest block number when the user explicitly tests the connection. The endpoint is not exposed to the frontend or stored in SQLite.

This is read-only connectivity only. Wallet balances, ERC-20 metadata, DEX quotes, transaction simulation, signing, and trading are not yet implemented. No private-key test is performed during RPC setup.

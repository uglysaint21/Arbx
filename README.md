# ArbX

**ArbX** is a locally operated desktop application in development for monitoring and, eventually, executing token-pair arbitrage opportunities across decentralised exchanges (DEXs) on the same blockchain. The intended release is a standalone Windows application.

> **Current status:** The Tauri desktop foundation, local authentication, SQLite strategy persistence, Windows Credential Manager wallet storage, Polygon PoS read-only RPC connectivity, and read-only POL/ERC-20 balance retrieval have implementation in the repository. DEX quotes, opportunity detection, atomic transaction simulation, signing, and trading are not implemented. Windows build/CI and live wallet-balance acceptance still require confirmation; see the [development log](docs/DEVELOPMENT_LOG.md).

## Product direction

The intended product includes:

- A locally running, password-protected desktop application.
- Local protection of wallet credentials.
- User-configured token-pair strategies.
- Same-chain price comparison across supported DEX venues.
- Profitability evaluation after applicable execution costs.
- A dashboard for portfolio balance, P&L, and active strategies.
- Trade history distinguishing expected from realised outcomes.
- A dark, professional visual identity with typewriter-inspired typography, orange and white accents, frosted-glass panels, and restrained animations.

Version 1 is scoped to **Polygon PoS (EVM, chain ID 137)**. The product specification names Uniswap, PancakeSwap, and SushiSwap as initial DEX examples; exact Polygon deployments and supported execution routes still need to be selected and verified.

The intended arbitrage cycle buys the target token on one DEX and sells it back into the starting token on another. Both legs must execute atomically in one transaction. Two independent swap transactions are not an acceptable substitute.

## Documentation

- [Product Specification](docs/PRODUCT_SPECIFICATION.md) — product behaviour, user flows, trading intent, visual direction, boundaries, and open product decisions.
- [Architecture](docs/ARCHITECTURE.md) — technical structure, system boundaries, security model, data concepts, testing approach, and build phases.
- [Development Guide](docs/DEVELOPMENT.md) — prerequisites and commands for running/building the application.
- [Development Log](docs/DEVELOPMENT_LOG.md) — chronological record of product clarification and engineering milestones, including verification caveats.
- [Roadmap](docs/ROADMAP.md) — phased plan from current verification through DEX data, simulation, paper trading, and controlled execution.
- [Wiki-ready pages](docs/wiki/Home.md) — version-controlled wiki entry points and page mirrors.

The repository documentation is the source of truth for the current project status. The files under `docs/wiki/` are wiki-ready mirrors; they are not themselves the GitHub-hosted Wiki pages.

## Current implementation

### Implemented in the repository

- Tauri 2 + React + TypeScript + Rust desktop application foundation.
- First-launch password setup, login, manual lock, and 15-minute inactivity lock.
- Argon2id password hashing and local SQLite persistence.
- Strategy create/list/delete persistence with native validation.
- Windows-native wallet import/removal using Windows Credential Manager; private-key material is not exposed to React or SQLite.
- User-configured Polygon PoS JSON-RPC endpoint stored in Windows Credential Manager.
- Read-only chain ID/block health checks.
- Read-only POL balance and selected ERC-20 symbol, decimals, and balance reads.

### Not yet implemented

- DEX adapters and live quote retrieval.
- Cross-venue opportunity detection and net-profit calculation.
- Atomic route construction and transaction simulation.
- Paper trading.
- Dedicated transaction signing, approvals, transaction broadcast, or live trading.
- Live portfolio valuation, P&L, and completed-trade history; dashboard figures remain illustrative.

### Verification status

Code being present does not mean it has passed acceptance. The latest workflow run inspected during the documentation update was cancelled, not successful. A successful Windows build, Rust test run, real-wallet import test, and live POL/ERC-20 balance verification have not been confirmed. The next checkpoint is to complete those checks before starting DEX integration.

## Security boundaries

- Never commit or share private keys, seed phrases, or credential-bearing RPC endpoints.
- Keep wallet secrets and RPC credentials inside the native secure-storage boundary.
- Keep read-only connectivity separate from transaction signing and execution.
- Do not enable live transaction submission until atomicity, quote validation, simulation, risk limits, transaction lifecycle handling, and security review are complete.

## Development workflow

Development milestones and verification evidence should be recorded in [docs/DEVELOPMENT_LOG.md](docs/DEVELOPMENT_LOG.md), and planned work should be updated in [docs/ROADMAP.md](docs/ROADMAP.md). Do not mark a phase complete until its stated verification gates pass.

# ArbX

**ArbX** is a planned locally operated desktop application for monitoring and executing token-pair arbitrage opportunities across decentralised exchanges (DEXs) on the same blockchain.

The intended application will let a user configure strategies, monitor prices across selected DEX venues, evaluate opportunities after transaction costs, and review portfolio performance and completed trades through a graphical dashboard. The long-term target is a standalone Windows application distributed as an `.exe`.

> **Project status:** Product definition and planning. The repository is being prepared before architecture and implementation are defined. No working trading engine or executable is available yet.

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

The product specification intentionally does not define the software architecture or implementation. Those will be documented separately after the remaining product decisions are clarified.

## Current development stage

The immediate next stage is to define and agree on the architecture. Implementation should begin only after that architecture is documented and reviewed.

Until then, this repository is the project’s source of truth for agreed product documentation. Work should be committed to the `main` branch unless the project workflow is deliberately changed.

## Scope and safety notes

ArbX is intended to interact with cryptocurrency wallets and execute on-chain transactions. Any eventual implementation must treat wallet credentials and transaction execution as security-critical. The product specification records the intended user experience; it is not a security audit, trading guarantee, or implementation claim.

## Repository

Source code and project documentation: [github.com/uglysaint21/Arbx](https://github.com/uglysaint21/Arbx)

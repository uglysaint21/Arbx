# ArbX Wiki

Welcome to the ArbX project documentation.

## Project overview

ArbX is intended to become a locally operated desktop application for evaluating and executing same-chain token-pair arbitrage across decentralised exchanges. Version 1 is scoped to Polygon PoS (chain ID 137). The product requires both legs of an arbitrage cycle to execute atomically in one transaction.

## Pages

- [Development log](Development-Log) — project origin, product clarification, engineering milestones, verification record, and development discipline.
- [Roadmap](Roadmap) — phased implementation plan, status labels, gates, and open decisions.
- [Product specification](../PRODUCT_SPECIFICATION.md) — user-facing behaviour and visual direction.
- [Architecture](../ARCHITECTURE.md) — technical structure and system boundaries.
- [Development guide](../DEVELOPMENT.md) — prerequisites and commands.

The development log and roadmap are maintained in the repository under `docs/`. This `docs/wiki/` directory is a version-controlled, wiki-ready mirror; it is not the GitHub-hosted Wiki database itself.

## Current checkpoint

The desktop foundation, local authentication, strategy persistence, Windows wallet credential boundary, Polygon read-only RPC checks, and read-only POL/ERC-20 balance retrieval have implementation in the repository. Build/CI and live wallet-balance verification must be confirmed before those integrations are treated as accepted. DEX quotes, atomic simulation, signing, and live trading remain future work.

## Safety

Never commit or share private keys, seed phrases, or credential-bearing RPC endpoints. Read-only wallet and RPC features must remain separate from transaction signing and execution.

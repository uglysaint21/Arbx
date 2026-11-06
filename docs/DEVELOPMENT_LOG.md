# ArbX Development Log

This log records the product and engineering milestones that are evidenced by the repository and the development work to date. It distinguishes implemented functionality from work that is planned or awaiting verification.

## Project origin and product definition

### 1. Initial product brief

ArbX began as a concept for a locally operated desktop application that monitors token-pair price differences across decentralised exchanges and, eventually, executes profitable same-chain arbitrage. The intended experience includes strategy management, a portfolio/P&L dashboard, trade history, local wallet protection, and a dark technical visual identity.

The initial product direction named Uniswap, PancakeSwap, and SushiSwap as DEX examples. These names are examples, not a claim that every venue is deployed or currently supported on Polygon.

### 2. Product behaviour clarified

The product brief was expanded into `docs/PRODUCT_SPECIFICATION.md`. The specification records the intended user experience and trading behaviour without mixing in implementation architecture.

Key clarified product constraints include:

- Version 1 is scoped to Polygon PoS (EVM, chain ID 137).
- A strategy is based on a starting token and a target token.
- A strategy compares at least two DEX venues on the same chain.
- The intended arbitrage cycle buys the target token on one venue and sells it back into the starting token on another.
- The two legs must execute atomically in one transaction; two separately submitted swaps do not meet the product requirement.
- A price difference alone is not sufficient: the decision must account for transaction costs and net profitability.
- Portfolio, P&L, strategy and history views are part of the intended product experience.

Several decisions remain open in the product specification, including exact Polygon DEX deployments and execution routes, funding assumptions, complete cost model, minimum profit and safety margin, trade-size/risk limits, valuation currency, and in-progress trade behaviour.

### 3. Architecture separated from product behaviour

A separate `docs/ARCHITECTURE.md` was introduced so product behaviour and implementation design remain distinct. It records the confirmed technology and system boundaries, security model, data concepts, testing approach, and implementation phases. Architecture details should be updated when decisions are made; unresolved product questions must not be silently converted into assumptions.

## Engineering milestones

### 4. Desktop application foundation

The repository contains a Tauri 2 desktop application with a React and TypeScript interface and a Rust native core.

The implemented local foundation includes:

- First-launch application password setup and login.
- Argon2id password hashing with random salts; plaintext application passwords are not stored.
- Explicit application locking and automatic locking after 15 minutes of inactivity.
- SQLite storage in the operating-system per-user application-data directory.
- Versioned database migration tracking and local strategy persistence.
- Native validation of strategy token addresses, distinct-token requirements, and duplicate DEX venue selections.

The browser preview is not equivalent to the native desktop application and does not provide the native credential-store or full local application behaviour.

### 5. Wallet credential boundary

A Windows-native wallet import/removal flow was added.

- Wallet import uses a native Windows credential prompt rather than a React password input.
- EVM public-address derivation takes place in Rust.
- The private key is stored in the current Windows user's Credential Manager through the keyring integration.
- The interface and SQLite metadata use the public address, not the private key.
- Wallet replacement/removal includes recovery handling for metadata persistence failures.
- The inactivity lock is also enforced by native protected commands.

A real user wallet import/acceptance test has deliberately not been claimed as complete. Never paste a private key or seed phrase into chat, an issue, a commit, or a log.

### 6. Polygon PoS read-only RPC connectivity

The Settings flow supports a user-configured Polygon PoS JSON-RPC endpoint.

- Endpoint entry uses a native Windows credential prompt.
- The endpoint, including any provider credential embedded in it, is stored in Windows Credential Manager rather than React state or SQLite.
- HTTPS is required for remote providers; HTTP is restricted to localhost/loopback.
- The native RPC client applies timeouts, response-size bounds, and JSON-RPC envelope validation.
- The connection test verifies chain ID 137 and reads the latest block number.
- The UI displays connection status and public chain/block information, not the endpoint.

This is read-only connectivity. It does not imply DEX access or transaction execution.

### 7. Read-only POL and ERC-20 balance retrieval

A subsequent implementation slice added a native authenticated balance-read command and Settings UI.

- Reads the imported wallet's POL balance.
- Supports a user-supplied ERC-20 contract address and reads its symbol, decimals, and balance.
- Rechecks Polygon chain ID 137, validates addresses, and checks that the token address contains contract code.
- Uses integer arithmetic for token-unit formatting rather than floating-point conversion.
- Clears displayed balances when the wallet is removed/replaced or the workspace locks.
- Does not sign, approve, construct, simulate, or broadcast transactions.

The implementation work is associated with [issue #1](https://github.com/uglysaint21/Arbx/issues/1) and [draft PR #2](https://github.com/uglysaint21/Arbx/pull/2). PR #2 must be checked for its current state before treating it as merged. A code change or a draft PR is not proof of a successful Windows build or live RPC acceptance test.

## Verification record and current caveats

- Unit tests were added for big-integer quantity parsing, exact token-unit formatting, EVM address validation, ERC-20 symbol ABI decoding, and quantity parsing.
- A Windows CI workflow was added to run the frontend build and Rust tests.
- The latest workflow run inspected during this documentation update was **cancelled**, not successful. Therefore this log does not claim that CI, the Windows desktop build, or live wallet balance retrieval has passed.
- The real-wallet import and live Polygon balance acceptance checks remain owner-run verification steps.
- Dashboard portfolio/P&L figures and chart remain illustrative until connected to real portfolio and trade data.
- DEX quote retrieval, route comparison, paper trading, atomic transaction simulation, signing, transaction submission, and the arbitrage engine are not yet complete.

## Development discipline

1. Keep private keys, seed phrases, RPC URLs containing credentials, and other secrets out of source control and chat.
2. Keep wallet secrets and RPC credentials inside the native secure-storage boundary.
3. Keep balance/quote discovery read-only until separately reviewed signing and execution work is approved.
4. Do not enable live trading until route validation, atomicity, simulation, risk limits, transaction lifecycle handling, and failure recovery are implemented and reviewed.
5. An arbitrage cycle must be atomic. Two independently submitted swap transactions are not an acceptable substitute.
6. Update this log and the roadmap when milestones actually change; mark verification separately from implementation.

# Roadmap

See the canonical, detailed roadmap in [docs/ROADMAP.md](../ROADMAP.md).

The immediate next gate is verification of the existing Windows build, Rust tests, Polygon RPC connection, and read-only POL/ERC-20 balance retrieval. DEX market-data adapters, profitability analysis, atomic route simulation, paper trading, signing, and controlled execution follow only after the preceding gates pass.

The roadmap is staged deliberately: no signing or live transaction submission should be introduced before atomicity, simulation, risk controls, failure handling, and security review are complete.

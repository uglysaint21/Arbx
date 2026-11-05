# ArbX Development Guide

## Current milestone

The repository contains the initial Tauri 2 + React + TypeScript + Rust desktop shell and the first native local-foundation slice.

### Implemented in the native core

- SQLite database created in the operating system's per-user application-data directory.
- Versioned schema migration ledger (migration 1) and initial settings/strategy tables.
- First-launch password setup with a minimum 12-character password.
- Passwords are stored as Argon2id password hashes with per-password random salts; plaintext passwords are not stored.
- Login and explicit application lock commands.
- Persistent Polygon PoS strategy create/list/delete commands.
- Native commands reject strategy access while the application is locked.
- Token address validation, distinct-token validation, and duplicate-venue rejection are performed again in Rust, not trusted solely to frontend validation.

### Still not implemented

- React login/setup/lock screens are not yet connected to these native commands.
- Windows Credential Manager / DPAPI-backed private-key storage and wallet import.
- Wallet address derivation and display.
- Automatic idle lock, password reset/recovery, and session timeout policy.
- Database backup/restore UI.
- Live Polygon RPC, DEX quotes, signing, and trading.

The existing dashboard remains a preview UI. Its metrics and chart are illustrative. The browser preview does not provide the native SQLite/password functionality.

## Prerequisites

For Windows desktop development, install Node.js LTS and npm, Rust using rustup with the MSVC toolchain, Microsoft C++ Build Tools with the Desktop development with C++ workload, and Microsoft Edge WebView2 Runtime.

See the official Tauri prerequisites: https://v2.tauri.app/start/prerequisites/

## Run in a browser

    npm install
    npm run dev

Open the Vite URL shown in the terminal (normally http://localhost:1420).

## Run as a desktop window

    npm install
    npm run tauri dev

## Build a Windows installer

On a configured Windows development machine:

    npm install
    npm run tauri build

The current bundle target is NSIS. See the official Tauri Windows installer guide: https://v2.tauri.app/distribute/windows-installer/

## Project safety rules

- Do not commit private keys, seed phrases, RPC secrets, or credential-bearing environment files.
- Do not add live transaction submission until quote validation, route simulation, risk limits, and transaction lifecycle handling are implemented and reviewed.
- The intended arbitrage is a two-leg same-chain cycle executed atomically in one transaction. Two separately submitted swaps do not satisfy that requirement.
- Keep chain-specific and DEX-specific details behind native integration adapters.

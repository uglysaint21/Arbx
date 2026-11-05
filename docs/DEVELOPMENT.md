# ArbX Development Guide

## Current milestone

The repository contains the initial Tauri 2 + React + TypeScript + Rust desktop shell.

This milestone is a UI preview only:
- Dashboard, navigation, strategy creation, History, and Settings surfaces are present.
- Portfolio figures and chart are illustrative fixture data.
- Polygon PoS is shown as the initial target network (chain ID 137).
- Strategy data exists only in frontend memory and is lost when the app reloads.
- Demo venues are placeholders, not connected DEX deployments.
- The bot toggle changes preview UI state only. It does not start a scanner.
- Wallet import, password authentication, SQLite, RPC access, quotes, signing, and trading are not implemented.
- Never enter a real private key into this preview.

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

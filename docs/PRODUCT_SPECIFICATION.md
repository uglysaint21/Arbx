# ArbX — Product Specification

> **Document purpose:** Define what ArbX is intended to do, how a user interacts with it, the trading logic it should follow, and the visual experience it should provide.
>
> **Scope boundary:** This document describes product behaviour and presentation only. It does **not** define the software architecture, repository structure, technology stack, modules, APIs, database design, deployment pipeline, or implementation plan. Those belong in a separate architecture document.

---

## 1. Product Overview

ArbX is a locally running desktop application for monitoring and executing token-pair arbitrage opportunities across decentralised exchanges (DEXs).

The application is intended to:

- Run on the user's own computer as a standalone application, ultimately distributed as a Windows executable (`.exe`).
- Allow the user to configure and manage arbitrage strategies.
- Monitor selected token pairs across supported DEXs on the same blockchain.
- Identify opportunities where the expected result of a trade is profitable after applicable transaction costs.
- Execute a trade only when the profitability condition is satisfied.
- Show portfolio balance, profit and loss (P&L), active strategies, and a record of completed trades.

ArbX is intended to be operated through a graphical desktop interface. The user should not need to interact with a command line during normal use.

## 2. Core Product Concepts

### 2.1 Token pair

A strategy operates on a pair of tokens. The system compares the value or exchange rate of that pair across DEXs on a single blockchain.

For clarity, a pair should be represented as:

- **Base token:** the token being exchanged from.
- **Quote token:** the token being exchanged into.

The exact convention for displaying the pair (for example, `TOKEN A / TOKEN B`) should be consistent throughout the application.

### 2.2 Blockchain

A blockchain is the network on which the token pair and the DEXs operate. ArbX compares DEXs only when they are on the **same blockchain** for a given arbitrage strategy.

The user chooses the blockchain when creating a strategy. The available choices should be presented in a drop-down list containing only chains supported by the application.

### 2.3 Decentralised exchange (DEX)

A DEX is a venue on which the selected token pair can be priced and traded. The initial examples named in the source specification are:

- Uniswap
- PancakeSwap
- SushiSwap

A strategy compares the selected pair across multiple DEXs. The examples in the original brief describe comparisons such as one DEX versus another, rather than a strategy using only one exchange.

### 2.4 Strategy

A strategy is a saved configuration that tells ArbX which token pair, blockchain, and DEX venues to monitor for arbitrage.

Each strategy is independently visible on the dashboard and can be added or deleted by the user.

### 2.5 Portfolio and P&L

- **Portfolio balance** shows the value or balance of the assets managed or tracked by ArbX.
- **P&L** shows the profit or loss over time and includes a simple line graph.

The precise valuation currency and whether balances are live, cached, or refreshed on demand are not defined in the original brief and remain product decisions.

## 3. User Experience and Application Flow

### 3.1 First launch and application password

On first launch, ArbX should present a password setup screen.

1. The user is asked to create a new application password.
2. The user confirms the password.
3. The application stores the password securely using an appropriate local password-protection mechanism. The password must not be stored as readable plain text.
4. After successful setup, the user can enter the application using that password.

On later launches, the application should show a password-entry screen before opening the main dashboard.

The application password protects access to ArbX. It is distinct from the wallet credentials used to authorise blockchain transactions.

### 3.2 Main dashboard

After successful authentication, the user lands on the main dashboard.

The dashboard should provide:

- A **Settings** control in the top-left area.
- A **History** control beside Settings.
- A prominent **ON/OFF** control in the top-right area for starting or stopping the bot.
- A **Portfolio** tab or panel showing the portfolio balance.
- A **P&L** tab or panel showing profit/loss and a simple line graph.
- A strategy area below the portfolio and P&L area, listing the user's configured strategies.

The ON/OFF control should make the bot's operating state clear. When switched off, ArbX should stop monitoring for and initiating new trades. The exact handling of a trade that is already in progress when the bot is switched off is not specified and must be decided separately.

### 3.3 Strategy area when no strategies exist

If the user has not created any strategies, the strategy area should display an empty state:

- Show the word **“Empty”**.
- Show a clearly visible **plus (+) icon**.
- The plus icon is the entry point for creating a strategy.

The empty state should be visually intentional and should make it obvious how to add the first strategy.

### 3.4 Adding a strategy

Selecting the plus icon should open a separate strategy-creation window or dialog.

The form should allow the user to:

1. Enter or paste the relevant token address or addresses.
2. Select the blockchain from a drop-down list.
3. Select the DEX venue or venues from a drop-down list.
4. Select **Proceed** to submit the configuration.

The application should validate the supplied information before accepting the strategy. At minimum, it should check that the required fields have been supplied and that the selected configuration is supported.

If the strategy is accepted:

- The new strategy is saved.
- The strategy appears in the dashboard's strategy area.
- A small success notification appears to confirm that the strategy was added.

If the strategy cannot be added, the application should explain the problem and leave the user able to correct the form. The exact validation rules and error messages are to be defined during product refinement.

### 3.5 Viewing and deleting strategies

Each saved strategy should appear as its own selectable item or tab in the strategy area.

The user should be able to:

- Select a strategy to view its details or monitoring status.
- Delete a strategy that is no longer needed.

Deleting a strategy should remove it from the active strategy list. Whether its historical trades remain in History should be preserved: deleting a strategy must not silently erase the record of trades already made.

The precise confirmation interaction for deletion is not specified.

## 4. Settings

The Settings control is located at the top-left of the main dashboard. It opens the application's settings area.

The settings area contains the following sections.

### 4.1 Change application password

The password-change flow should be:

1. Ask the user to enter the current application password.
2. Ask the user to confirm or verify the current password.
3. Allow the user to enter a new password.
4. Ask the user to confirm the new password.
5. Change the application password only if the current password is valid and the new-password entries match.

The application should provide clear feedback if the current password is incorrect or the new entries do not match.

### 4.2 Change wallet

The wallet section should allow the user to provide the wallet information required for ArbX to carry out transactions. The original brief specifically mentions a MetaMask wallet private key or other necessary wallet details.

- The input should initially be empty rather than displaying a saved private key.
- After the user supplies the required wallet credentials, ArbX should protect them using local encryption and store them locally.
- When a transaction requires wallet authorisation, ArbX should be able to retrieve and use the stored credentials securely.
- The interface should not expose the saved private key as ordinary readable text.

Wallet credentials and the application password serve different purposes and should be treated as separate credentials.

**Important product-security expectation:** wallet credentials control funds. The interface should make it clear when credentials are being added or changed. The exact wallet connection and signing experience is not defined in the original brief.

### 4.3 About

The About section should explain, in plain language:

- What ArbX does.
- What an arbitrage strategy is in the context of the application.
- Which chains and DEXs are supported.
- How ArbX evaluates whether an opportunity is profitable.
- What the ON/OFF control does.

It should help a user understand the application's purpose and normal operation without requiring technical knowledge.

## 5. Trade History

The History control sits beside Settings on the main dashboard. Selecting it opens the History view.

History should contain a record of trades made by ArbX. Each record should show, where the information is available:

- The date and time of the trade.
- The strategy and token pair involved.
- The blockchain used.
- The DEX or DEXs involved.
- The expected outcome at the time the trade was considered.
- The actual outcome after execution.
- Whether the trade ultimately resulted in a gain or a loss.

The record should distinguish the expected result from the realised result. A trade should not be labelled profitable solely because it was expected to be profitable before execution.

The original brief does not specify filters, sorting, export, transaction-hash display, or the exact monetary fields. These are not assumed requirements in this document.

## 6. Arbitrage and Trading Logic

This section describes the intended user-visible trading behaviour, not the implementation of the trading engine.

### 6.1 Scope of an arbitrage strategy

- ArbX works with **token pairs only**.
- A strategy compares the same token pair across different DEXs.
- The DEXs being compared must operate on the same blockchain for that strategy.
- The initial DEX examples are Uniswap, PancakeSwap, and SushiSwap.
- The application should consider only configurations supported by the selected chain and available DEX venues.

ArbX is not described as a cross-chain arbitrage application. Moving assets between different blockchains is outside the stated scope.

### 6.2 Opportunity detection

For each active strategy, ArbX should monitor the selected token pair across the DEXs configured for that strategy.

It should compare the available prices or exchange rates and identify a possible arbitrage route when the difference between venues may produce a favourable result.

The system should consider the relevant trade costs before treating a price difference as a valid opportunity.

### 6.3 Profitability check

A price difference alone is not sufficient reason to trade.

Before executing a swap or trade, ArbX must estimate the amount the user would receive at the end of the proposed trade and account for applicable transaction costs.

A trade should proceed only when the estimated final value or amount represents a profit after those costs.

At a minimum, the profitability decision must account for transaction fees. The original brief does not define the complete list of costs, the minimum acceptable profit, or the safety margin required to account for price movement. Those details remain open decisions.

### 6.4 Trade execution

The original brief describes the intended flow as follows:

1. ArbX identifies a potential opportunity for a configured token pair across DEXs on the same chain.
2. It determines the relevant DEX prices and the proposed route.
3. It uses the available deposited crypto to obtain the token required for the strategy, favouring the lowest relevant price among the configured DEXs.
4. It checks whether the expected final amount remains profitable after transaction costs.
5. It proceeds with the trade only if the profitability condition is met.
6. After the trade, it updates History with the trade details and outcome.

The precise sequence of swaps, the meaning of “lowest price” for each side of the pair, and whether a trade is one swap or a sequence of swaps are not fully specified in the original brief. This description therefore captures the stated intent without imposing a more detailed execution model.

### 6.5 After execution

After a trade has been attempted, ArbX should update the relevant user-facing information:

- Add the trade to History.
- Record the DEXs and token pair involved.
- Record the expected and actual result.
- Indicate whether the realised outcome was a gain or loss.
- Reflect the resulting asset balances and P&L in the dashboard.

If a proposed trade is rejected by the profitability check, it should not be represented as a completed trade. Whether rejected opportunities should be shown in a separate activity log is not specified.

## 7. Visual Design and Interaction

### 7.1 Overall visual direction

ArbX should have a professional, cohesive interface with a predominantly dark theme and a subtle vintage/technical character.

The intended visual direction combines:

- A dark background.
- A typewriter-inspired font.
- A restrained vintage feel.
- Frosted-glass styling for tabs, panels, and icons, inspired by the liquid-glass appearance of Apple interfaces.
- High-contrast, easily readable text.
- Tasteful use of orange shades and white for typography and emphasis.

The result should feel like a polished desktop trading tool rather than a generic dashboard or a playful game interface.

### 7.2 Colour and readability

- Use a dark base palette throughout the application.
- Use shades of orange as accent colours for emphasis, selected states, and important controls.
- Use white and suitably muted light tones for readable text.
- Maintain strong contrast between text, controls, and their backgrounds.
- Do not allow glass effects, transparency, or decorative styling to reduce legibility.

### 7.3 Typography

Use a typewriter-like font treatment to establish the application's character. Typography must remain clear and readable at normal desktop viewing sizes.

The typewriter style should be applied consistently rather than mixed with unrelated font styles.

### 7.4 Panels, tabs, and icons

Tabs, panels, and icons should have a frosted-glass appearance. Their boundaries and active states must remain clear against the dark background.

Icons should be consistent in style, scale, and visual weight. Their meaning should be recognisable and their interactive areas easy to use.

### 7.5 Motion and interaction feedback

Use subtle animations for:

- Hovering over tabs and icons.
- Clicking controls.
- Selecting or switching tabs.
- Showing success or error notifications.

Animations should feel smooth and restrained. They should provide feedback without distracting from portfolio information, strategy status, or trading activity.

### 7.6 Consistency

The colour palette, typography, iconography, panel treatments, spacing, and animations should feel like parts of one coherent design system. Professionalism and readability take priority over decorative effects.

## 8. Functional Boundaries

The following are explicit boundaries of the product as currently described:

- ArbX is a desktop application intended to run locally on the user's machine.
- The intended final deliverable is an executable application (`.exe`).
- The application has a password-protected entry flow.
- Wallet credentials are stored locally in protected form.
- Strategies are based on token pairs.
- Arbitrage comparisons take place across DEXs on the same blockchain.
- A trade is executed only when the estimated final result is profitable after transaction fees.
- The dashboard presents portfolio balance, P&L, and configured strategies.
- Completed trades are recorded in History.
- The interface follows the dark, typewriter-inspired, frosted-glass visual direction described above.

This document does not define how these behaviours are implemented.

## 9. Open Product Decisions

The original brief establishes the product direction but leaves several details unresolved. These should be decided before the architecture and implementation specifications are finalised.

1. **Supported blockchains:** Which exact chains are supported at launch? Polygon and Solana appear as examples, but the complete list is not provided.
2. **DEX availability by chain:** Which of Uniswap, PancakeSwap, and SushiSwap are available on each supported chain? The interface should not imply that every DEX exists on every chain.
3. **Strategy inputs:** Does a strategy require two token addresses, or does the user provide one token address and select the paired token elsewhere?
4. **DEX selection:** Must the user select at least two DEXs for each strategy? The comparison examples imply multiple venues, but the form description mentions selecting “the exchange” in the singular.
5. **Trade model:** Is the intended arbitrage a buy on one DEX and sell on another, or does the strategy involve an initial conversion followed by a separate route? The exact sequence needs to be confirmed.
6. **Funding model:** What does “deposited crypto” mean in the product? Is ArbX using assets already held in the configured wallet, and which token is used as the starting asset?
7. **Profit calculation:** In which currency or token is profit measured? Which costs must be included beyond transaction fees, and what minimum net profit is required?
8. **Execution protection:** Should the user be able to set a maximum trade size, minimum profit threshold, or other limits? These controls are not specified in the original brief.
9. **Bot state:** Should the bot start in the OFF state every time the application opens, or remember its previous state?
10. **In-progress trades:** What should happen if the user switches the bot off while a trade is being prepared or submitted?
11. **Portfolio valuation:** Which currency should be used to display portfolio balance and P&L?
12. **History detail:** Which exact values should each trade record contain, and should failed or rejected attempts be recorded separately?
13. **Wallet interaction:** Should the user paste a private key, connect a wallet through another method, or have both options? The original brief mentions MetaMask details but does not settle the interaction.
14. **Password recovery:** What should happen if the user forgets the application password? No recovery process is specified.
15. **Strategy deletion:** Should the user be asked to confirm before deleting a strategy? Historical trade records should remain available, but this should be confirmed as the intended behaviour.

These are product questions, not architecture decisions. They are listed explicitly so that later documents do not silently make assumptions on the user's behalf.

---

## 10. Product Summary

ArbX is a locally operated desktop arbitrage application. The user protects access with an application password, configures a wallet, and creates strategies for token pairs on selected blockchains and DEXs. The application monitors the configured venues on the same chain, evaluates whether a proposed trade is profitable after transaction costs, and executes only when that condition is met. The dashboard presents portfolio balance, P&L, and active strategies, while History records completed trades and their expected and realised outcomes.

Its visual identity is dark, professional, and vintage-technical: typewriter-inspired typography, orange and white accents, frosted-glass panels and icons, and restrained interaction animations.

**Architecture and implementation are intentionally left for a separate document.**

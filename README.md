# Memecoin Prediction Market

This project implements a prediction market for meme coin prices using Anchor, a framework for Solana programs. Users can create markets for specific meme coins, make predictions on whether the price will go up or down within a specified timeframe, and settle markets based on the actual outcome.

## Scope

Develop a program using Anchor to create a prediction market for Solana meme coins. Users should be able to create markets for specific meme coins, make price predictions, and settle markets based on the actual price movement.

## Features

1. **Market Creation**: Allows users to create prediction markets for specific meme coins.
2. **Placing Bets**: Enables users to place bets on whether the price will go up or down.
3. **Market Settlement**: Settles markets based on the actual price movement at the expiry time.
4. **Winning Claims**: Allows winners to claim their winnings after market settlement.

## Prerequisites

- Rust
- Solana CLI
- Node.js
- Yarn
- Anchor

## Installation

1. **Install Solana CLI**:
    ```sh
    sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
    ```

2. **Install Rust**:
    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

3. **Install Node.js and Yarn**:
    ```sh
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
    nvm install --lts
    npm install --global yarn
    ```

4. **Install Anchor**:
    ```sh
    cargo install --git https://github.com/project-serum/anchor --tag v0.24.2 anchor-cli --locked
    ```

## Setup

1. Clone the repository:
    ```sh
    git clone git@github.com:0xCipherCoder/memecoin_prediction_market.git
    cd memecoin_prediction_market
    ```

2. Install dependencies:
    ```sh
    npm install
    anchor build
    ```

3. Deploy the programs to Solana Local Testnet:
    ```sh
    anchor deploy
    ```

## Usage

### Building the Program

1. Build the Solana program:
    ```sh
    anchor build
    ```

2. Deploy the program to your local Solana cluster:
    ```sh
    anchor deploy
    ```

### Running Tests

1. Run the tests:
    ```sh
    anchor test
    ```

2. Ensure your local Solana test validator is running:
    ```sh
    solana-test-validator
    ```

### Test Report

```sh
anchor test

Finished release [optimized] target(s) in 0.13s
Found a 'test' script in the Anchor.toml. Running it as a test suite!
Running test suite: "/home/pradip/Cipher/OpenSource/memecoin_prediction_market/Anchor.toml"

  memecoin_prediction_market
    ✔ Initializes the market (443ms)
    ✔ Places a bet (430ms)
    ✔ Fails to place a bet after expiry (5022ms)
    ✔ Settles the market (249ms)
    ✔ Claims winnings (433ms)
    ✔ Fails to claim winnings twice


  6 passing (9s)
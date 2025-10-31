# Kairo: A Stylus Smart Contract Library

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/Team-Oracle/kairo)
[![Stylus](https://img.shields.io/badge/Built%20for-Arbitrum%20Stylus-blueviolet.svg)](https://arbitrum.io/)

Kairo is a production-ready smart contract library for the [Arbitrum Stylus VM](https://arbitrum.io/), inspired by OpenZeppelin. It provides secure, gas-efficient, and reusable components for common token standards, written natively in Rust.

---

## 🚩 The Problem

The Arbitrum Stylus environment is revolutionary, allowing developers to write contracts in high-performance languages like Rust. However, this new ecosystem presents a "blank page" problem: without a standard library, every developer must "reinvent the wheel" by writing their own ERC20 or NFT contracts from scratch. This is slow and a leading cause of security vulnerabilities.

**Kairo solves this by providing a foundational, trusted, and battle-tested set of contracts for developers to build upon.**

---

## ✨ Features

- **ERC20:** Full-featured implementation of the fungible token standard.
- **ERC721:** Full-featured implementation of the non-fungible token (NFT) standard.
- **Written in Rust:** Leverages Rust's safety and performance for the Stylus VM.
- **Reusable & Composable:** Built as a true library, making it easy to import and extend contracts.

---

## 🚀 Getting Started

Build and deploy your own custom ERC20 token in minutes!

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [`cargo-stylus`](https://github.com/OffchainLabs/stylus)
- [`foundry`](https://book.getfoundry.sh/getting-started/installation)
- Wallet with testnet funds

---

### 1. Create a New Stylus Project

```bash
cargo stylus new my-awesome-token
cd my-awesome-token
```

---

### 2. Add Kairo as a Dependency

Open your `Cargo.toml` and add Kairo to `[dependencies]`:

```toml
[dependencies]
# ... other dependencies
kairo = { git = "https://github.com/Team-Oracle/kairo.git", branch = "main" }
```

---

### 3. Write Your Custom Token Contract

Open `src/lib.rs` (or `src/main.rs`) and replace its contents:

```rust
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// 1. Import Kairo ERC20 components
use kairo::erc20::{Erc20, Erc20Params, Erc20Error};
use stylus_sdk::{alloy_primitives::U256, prelude::*};

// 2. Define token properties
struct MyTokenParams;

impl Erc20Params for MyTokenParams {
    const NAME: &'static str = "My Awesome Token";
    const SYMBOL: &'static str = "MAT";
    const DECIMALS: u8 = 18;
}

// 3. Define storage and inherit ERC20 logic
sol_storage! {
    #[entrypoint]
    struct MyAwesomeToken {
        #[borrow]
        Erc20<MyTokenParams> erc20;
    }
}

// 4. (Optional) Add custom functions (e.g., mint initial supply)
#[public]
#[inherit(Erc20<MyTokenParams>)]
impl MyAwesomeToken
where
    Self: HostAccess,
{
    pub fn init(&mut self) -> Result<(), Erc20Error> {
        let initial_supply = U256::from(1_000_000) * U256::from(10).pow(U256::from(MyTokenParams::DECIMALS));
        self.erc20.mint(self.vm().msg_sender(), initial_supply)
    }
}
```

---

### 4. Deploy to a Testnet

Use `cargo stylus` to deploy to Sepolia (or another supported testnet):

```bash
cargo stylus deploy --no-verify \
  -e <YOUR_SEPOLIA_RPC_URL> \
  --private-key <YOUR_WALLET_PRIVATE_KEY>
```

Your custom token is now live, inheriting all of Kairo's security and functionality!

---

## 🧑‍💻 Local Development & Testing

Want to contribute to Kairo? Run the test suite as follows.

### 1. Run a Local Node

In your first terminal:

```bash
# In the kairo/ project root
./run-dev-node.sh
```

### 2. Run the Test Suite

In a second terminal:

```bash
# In the kairo/ project root
./run_tests.sh
```

You'll see detailed test output, ending with a final success message.

---

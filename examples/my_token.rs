// examples/my_token.rs

// This cfg attribute is a must for all example files.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Import the ERC20 logic from the library we are building.
use stylus_library::erc20::{Erc20, Erc20Params, Erc20Error};
use stylus_sdk::{alloy_primitives::{Address, U256}, entrypoint, msg, prelude::*};

// Define the parameters for our specific token.
struct MyTokenParams;

impl Erc20Params for MyTokenParams {
    const NAME: &'static str = "My Awesome Token";
    const SYMBOL: &'static str = "MAT";
    const DECIMALS: u8 = 18;
}

// Define the storage layout for our contract.
sol_storage! {
    #[entrypoint] // The entrypoint is the top-level contract.
    struct MyToken {
        #[borrow] // This allows the ERC20 logic to access the contract's storage.
        Erc20<MyTokenParams> erc20;
    }
}

// Implement the public interface of our contract.
#[public]
#[inherit(Erc20<MyTokenParams>)] // Inherit all the functions from the ERC20 library implementation.
impl MyToken {
    /// Mints 1,000,000 tokens to the contract deployer.
    /// This is the constructor.
    pub fn init(&mut self) -> Result<(), Erc20Error> {
        let initial_supply = U256::from(1_000_000) * U256::from(10).pow(U256::from(MyTokenParams::DECIMALS));
        self.erc20.mint(msg::sender(), initial_supply)
    }

    // You could add other custom functions here if you wanted.
}
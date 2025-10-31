// examples/my_nft.rs

#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

use alloc::string::String;
use stylus_library::erc721::{Erc721, Erc721Params, Erc721Error};
use stylus_sdk::{alloy_primitives::{Address, U256}, entrypoint, msg, prelude::*};

// Define the parameters for our specific NFT collection.
struct MyNFTParams;

impl Erc721Params for MyNFTParams {
    const NAME: &'static str = "My Awesome NFT";
    const SYMBOL: &'static str = "MAN";

    fn token_uri(token_id: U256) -> String {
        // In a real project, this would point to a metadata server.
        alloc::format!("https://example.com/nft/{}", token_id)
    }
}

// Define the storage layout for our NFT contract.
// CORRECT VERSION for examples/my_nft.rs
sol_storage! {
    #[entrypoint]
    struct MyNFT {
        #[borrow]
        Erc721<MyNFTParams> erc721; // <-- This is now correct
    }
}

// Implement the public interface of our contract.
#[public]
#[inherit(Erc721<MyNFTParams>)]
impl MyNFT {
    // We don't need an `init` constructor here, but you could add one.
    
    /// Public minting function. Anyone can call this to get a new NFT.
    pub fn mint(&mut self, to: Address) -> Result<(), Erc721Error> {
        self.erc721.mint(to)
    }

    /// Public burning function. The owner of an NFT can call this to burn it.
    pub fn burn(&mut self, token_id: U256) -> Result<(), Erc721Error> {
        // This will automatically fail if the caller is not the owner.
        let owner = self.erc721.owner_of(token_id)?;
        self.erc721.burn(owner, token_id)
    }
}
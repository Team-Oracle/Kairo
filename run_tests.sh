#!/bin/bash

# Exit immediately if any command fails
set -e

# --- CONFIGURATION ---
DEV_KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
DEV_ADDRESS=0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E
RPC_URL="http://localhost:8547"

ORIGINAL_LIB_FILE="src/lib.rs"
BACKUP_LIB_FILE="src/lib.rs.bak"

EXPECTED_TOKEN_BALANCE="0x00000000000000000000000000000000000000000000d3c21bcecceda1000000"
EXPECTED_NFT_OWNER="0x0000000000000000000000003f1eae7d46d88f08fc2f8ed27fcb2ab183eb2d0e"

# --- HELPER & CLEANUP FUNCTIONS ---
assert_eq() {
    # This now uses a robust sed command to strip all ANSI color codes
    val1=$(echo "$1" | sed 's/\x1b\[[0-9;]*m//g' | tr '[:upper:]' '[:lower:]')
    val2=$(echo "$2" | sed 's/\x1b\[[0-9;]*m//g' | tr '[:upper:]' '[:lower:]')
    if [ "$val1" == "$val2" ]; then
        echo "✅ SUCCESS: Matched expected value."
    else
        echo "❌ FAILURE: Mismatch!"
        echo "   Expected: $2"
        echo "   Got:      $1"
        exit 1
    fi
}

cleanup() {
  echo ""
  echo "--- Cleaning up: Restoring original src/lib.rs ---"
  if [ -f "$BACKUP_LIB_FILE" ]; then
    mv "$BACKUP_LIB_FILE" "$ORIGINAL_LIB_FILE"
    echo "Restore complete."
  fi
}

trap cleanup EXIT

# --- SCRIPT START ---
echo "--- Starting Fully Automated End-to-End Tests ---"
echo "--- Backing up original src/lib.rs ---"
cp "$ORIGINAL_LIB_FILE" "$BACKUP_LIB_FILE"


# --- PART 1: DEPLOY AND TEST ERC20 TOKEN ---
echo ""
echo "--- Preparing src/lib.rs for ERC20 Deployment ---"
cat > "$ORIGINAL_LIB_FILE" << 'EOF'
// src/lib.rs (Temporary content for deploying MyToken)

#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

pub mod erc20;
pub mod erc721;

use crate::erc20::{Erc20, Erc20Params, Erc20Error};
use stylus_sdk::{alloy_primitives::U256, prelude::*};

struct MyTokenParams;

impl Erc20Params for MyTokenParams {
    const NAME: &'static str = "My Awesome Token";
    const SYMBOL: &'static str = "MAT";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[entrypoint]
    struct MyToken {
        #[borrow]
        Erc20<MyTokenParams> erc20;
    }
}

#[public]
#[inherit(Erc20<MyTokenParams>)]
impl MyToken
where
    Self: HostAccess,
{
    pub fn init(&mut self) -> Result<(), Erc20Error> {
        let initial_supply = U256::from(1_000_000) * U256::from(10).pow(U256::from(MyTokenParams::DECIMALS));
        self.erc20.mint(self.vm().msg_sender(), initial_supply)
    }
}
EOF

echo "--- Deploying ERC20 Token ---"
# We accept the colored output and clean it with sed
DEPLOY_OUTPUT_TOKEN=$(cargo stylus deploy --private-key $DEV_KEY)
# THE FIX IS HERE: `sed` command correctly strips all ANSI color codes
TOKEN_ADDRESS=$(echo "$DEPLOY_OUTPUT_TOKEN" | grep "deployed code at address" | awk '{print $5}' | sed 's/\x1b\[[0-9;]*m//g')
echo "Token deployed to new address: $TOKEN_ADDRESS"

echo "--- Testing ERC20 Token ---"
echo "[1/2] Initializing token..."
cast send $TOKEN_ADDRESS "init()" --private-key $DEV_KEY --rpc-url $RPC_URL > /dev/null
echo "[2/2] Checking initial balance..."
TOKEN_BALANCE=$(cast call $TOKEN_ADDRESS "balanceOf(address)" $DEV_ADDRESS --rpc-url $RPC_URL)
assert_eq $TOKEN_BALANCE $EXPECTED_TOKEN_BALANCE


# --- PART 2: DEPLOY AND TEST ERC721 NFT ---
echo ""
echo "--- Preparing src/lib.rs for ERC721 Deployment ---"
cat > "$ORIGINAL_LIB_FILE" << 'EOF'
#![cfg_attr(not(any(feature = "export_abi", test)), no_main)]
extern crate alloc;

use alloc::string::String;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

sol_storage! {
    #[entrypoint]
    pub struct MyNFT {
        mapping(uint256 => address) owners;
        mapping(address => uint256) balances;
        uint256 total_supply;
    }
}

#[external]
impl MyNFT {
    pub fn name(&self) -> Result<String, Vec<u8>> {
        Ok("My Awesome NFT".to_string())
    }
    pub fn symbol(&self) -> Result<String, Vec<u8>> {
        Ok("MAN".to_string())
    }
    pub fn owner_of(&self, token_id: U256) -> Result<Address, Vec<u8>> {
        let owner = self.owners.get(token_id);
        if owner.is_zero() {
            return Err(b"ERC721: invalid token ID".to_vec());
        }
        Ok(owner)
    }
    pub fn balance_of(&self, owner: Address) -> Result<U256, Vec<u8>> {
        Ok(self.balances.get(owner))
    }
    pub fn mint(&mut self, to: Address) -> Result<U256, Vec<u8>> {
        let new_token_id = self.total_supply.get();
        self.total_supply.set(new_token_id + U256::from(1));
        let mut to_balance = self.balances.setter(to);
        let old_balance = to_balance.get();
        to_balance.set(old_balance + U256::from(1));
        self.owners.insert(new_token_id, to);
        Ok(new_token_id)
    }
}
EOF

echo "--- Deploying ERC721 NFT ---"
# We accept the colored output and clean it with sed
DEPLOY_OUTPUT_NFT=$(cargo stylus deploy --private-key $DEV_KEY)
# THE FIX IS HERE: `sed` command correctly strips all ANSI color codes
NFT_ADDRESS=$(echo "$DEPLOY_OUTPUT_NFT" | grep "deployed code at address" | awk '{print $5}' | sed 's/\x1b\[[0-9;]*m//g')
echo "NFT deployed to new address: $NFT_ADDRESS"

echo "--- Testing ERC721 NFT ---"
echo "[1/2] Minting NFT (Token ID 0)..."
cast send $NFT_ADDRESS "mint(address)" $DEV_ADDRESS --private-key $DEV_KEY --rpc-url $RPC_URL > /dev/null
echo "[2/2] Checking owner of Token ID 0..."
NFT_OWNER=$(cast call $NFT_ADDRESS "ownerOf(uint256)" 0 --rpc-url $RPC_URL)
assert_eq $NFT_OWNER $EXPECTED_NFT_OWNER


# --- FINAL SUCCESS ---
echo ""
echo "🎉🎉🎉 ALL TESTS PASSED! 🎉🎉🎉"
// src/erc20.rs

extern crate alloc;

// Implementation of the ERC-20 standard
//
// The eponymous [`Erc20`] type provides all the standard methods,
// and is intended to be inherited by other contract types.
//
// You can configure the behavior of [`Erc20`] via the [`Erc20Params`] trait,
// which allows specifying the name, symbol, and decimals of the token.
//
// Note that this code is unaudited and not fit for production use.

use alloc::string::String;
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use core::marker::PhantomData;
use stylus_sdk::{evm, msg, prelude::*};

pub trait Erc20Params {
    const NAME: &'static str;
    const SYMBOL: &'static str;
    const DECIMALS: u8;
}

sol_storage! {
    pub struct Erc20<T> {
        mapping(address => uint256) balances;
        mapping(address => mapping(address => uint256)) allowances;
        uint256 total_supply;
        PhantomData<T> phantom;
    }
}

sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
}

#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

impl<T: Erc20Params> Erc20<T> {
    pub fn _transfer(&mut self, from: Address, to: Address, value: U256) -> Result<(), Erc20Error> {
        let mut sender_balance = self.balances.setter(from);
        let old_sender_balance = sender_balance.get();
        if old_sender_balance < value {
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance {
                from,
                have: old_sender_balance,
                want: value,
            }));
        }
        sender_balance.set(old_sender_balance - value);

        let mut to_balance = self.balances.setter(to);
        let old_to_balance = to_balance.get();
        to_balance.set(old_to_balance + value); // BORROW CHECKER FIX

        evm::log(Transfer { from, to, value });
        Ok(())
    }

    pub fn mint(&mut self, address: Address, value: U256) -> Result<(), Erc20Error> {
        let mut balance = self.balances.setter(address);
        let old_balance = balance.get();
        balance.set(old_balance + value); // BORROW CHECKER FIX

        let old_total_supply = self.total_supply.get();
        self.total_supply.set(old_total_supply + value);

        evm::log(Transfer {
            from: Address::ZERO,
            to: address,
            value,
        });
        Ok(())
    }

    pub fn burn(&mut self, address: Address, value: U256) -> Result<(), Erc20Error> {
        let mut balance = self.balances.setter(address);
        let old_balance = balance.get();
        if old_balance < value {
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance {
                from: address,
                have: old_balance,
                want: value,
            }));
        }
        balance.set(old_balance - value);

        let old_total_supply = self.total_supply.get();
        self.total_supply.set(old_total_supply - value);

        evm::log(Transfer {
            from: address,
            to: Address::ZERO,
            value,
        });
        Ok(())
    }
}

#[public]
impl<T: Erc20Params> Erc20<T> {
    pub fn name(&self) -> Result<String, Vec<u8>> {
        Ok(T::NAME.into())
    }
    pub fn symbol(&self) -> Result<String, Vec<u8>> {
        Ok(T::SYMBOL.into())
    }
    pub fn decimals(&self) -> Result<u8, Vec<u8>> {
        Ok(T::DECIMALS)
    }
    pub fn total_supply(&self) -> Result<U256, Vec<u8>> {
        Ok(self.total_supply.get())
    }
    pub fn balance_of(&self, owner: Address) -> Result<U256, Vec<u8>> {
        Ok(self.balances.get(owner))
    }
    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Vec<u8>> {
        match self._transfer(msg::sender(), to, value) {
            Ok(_) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        let mut sender_allowances = self.allowances.setter(from);
        let mut allowance = sender_allowances.setter(msg::sender());
        let old_allowance = allowance.get();
        if old_allowance < value {
            return Err(Erc20Error::InsufficientAllowance(InsufficientAllowance {
                owner: from,
                spender: msg::sender(),
                have: old_allowance,
                want: value,
            })
            .into());
        }
        allowance.set(old_allowance - value);
        match self._transfer(from, to, value) {
            Ok(_) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }
    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Vec<u8>> {
        self.allowances.setter(msg::sender()).insert(spender, value);
        evm::log(Approval {
            owner: msg::sender(),
            spender,
            value,
        });
        Ok(true)
    }
    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Vec<u8>> {
        Ok(self.allowances.getter(owner).get(spender))
    }
}
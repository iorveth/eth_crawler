use crate::errors::*;
use eth_address::address::is_address;

pub fn ensure_valid_eth_address(address: &str) -> Result<(), ServerError> {
    if !is_address(address.to_string()) {
        Err(ServerError::InvalidAddress {
            address: address.to_string(),
        })
    } else {
        Ok(())
    }
}

pub fn ensure_valid_starting_block_number(
    starting_block_number: u64,
    current_block_number: u64,
) -> Result<(), ServerError> {
    if starting_block_number > current_block_number {
        Err(ServerError::InvalidStartingBlockNumber {
            starting_block_number,
            current_block_number,
        })
    } else {
        Ok(())
    }
}

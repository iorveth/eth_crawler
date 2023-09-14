use crate::{errors::*, TransactionFormInput};
use chrono::{TimeZone, Utc};
use entity::transactions;
use serde_json::Value;

lazy_static! {
    static ref ETHERSCAN_API_KEY: String =
        std::env::var("ETHERSCAN_API_KEY").expect("ETHERSCAN_API_KEY is not set in .env file");
}

// API URL
pub const ETHERSCAN_API: &str = "https://api.etherscan.io/api";

// Modules
pub const MODULE_ACCOUNT: &str = "?module=account";
pub const MODULE_PROXY: &str = "?module=proxy";

// Actions
pub const ACTION_TXLIST: &str = "&action=txlist";
pub const ACTION_ETH_BLOCK_NUMBER: &str = "&action=eth_blockNumber";

// Placeholders
pub const API_KEY_PLACEHOLDER: &str = "&apikey=";
pub const ADDRESS_PLACEHOLDER: &str = "&address=";
pub const START_BLOCK_PLACEHOLDER: &str = "&startblock=";
pub const END_BLOCK_PLACEHOLDER: &str = "&endblock=";
pub const PAGE_PLACEHOLDER: &str = "&page=";
pub const OFFSET_PLACEHOLDER: &str = "&offset=";
pub const SORT_PLACEHOLDER: &str = "&sort=";

/// Get current eth block numberv
pub async fn get_current_block_number() -> Result<u64, ServerError> {
    let request = ETHERSCAN_API.to_string()
        + MODULE_PROXY
        + ACTION_ETH_BLOCK_NUMBER
        + API_KEY_PLACEHOLDER
        + &ETHERSCAN_API_KEY;

    let resp = reqwest::get(request).await?.json::<Value>().await?;
    let block_number_hex_str = resp["result"]
        .as_str()
        .ok_or(ServerError::ReqwestParsingError)?;

    let block_number = u64::from_str_radix(block_number_hex_str.trim_start_matches("0x"), 16)
        .map_err(|_| ServerError::ReqwestParsingError)?;

    Ok(block_number)
}

pub async fn parse_transactions(
    transactions: &[Value],
) -> Result<Vec<transactions::Model>, ServerError> {
    let mut parsed_transactions = vec![];
    for transaction in transactions.iter() {
        let tx_id = transaction["hash"]
            .as_str()
            .ok_or(ServerError::ReqwestParsingError)?;

        let address_from = transaction["from"]
            .as_str()
            .ok_or(ServerError::ReqwestParsingError)?;

        let address_to = transaction["to"]
            .as_str()
            .ok_or(ServerError::ReqwestParsingError)?;

        let value = transaction["value"]
            .as_str()
            .map(|value| value.parse::<u64>())
            .ok_or(ServerError::ReqwestParsingError)??;

        let gas_used = transaction["gasUsed"]
            .as_str()
            .map(|gas_used| gas_used.parse::<u64>())
            .ok_or(ServerError::ReqwestParsingError)??;

        let gas_price = transaction["gasPrice"]
            .as_str()
            .map(|gas_price| gas_price.parse::<u64>())
            .ok_or(ServerError::ReqwestParsingError)??;

        let block_number = transaction["blockNumber"]
            .as_str()
            .map(|block_number| block_number.parse::<u64>())
            .ok_or(ServerError::ReqwestParsingError)??;

        let timestamp = transaction["timeStamp"]
            .as_str()
            .map(|timestamp| timestamp.parse::<i64>())
            .ok_or(ServerError::ReqwestParsingError)??;

        let date_time = Utc.timestamp_opt(timestamp, 0).unwrap();

        println!("hash: {}", tx_id);

        let parsed_transaction = transactions::Model {
            tx_id: tx_id.to_string(),
            address_from: address_from.to_string(),
            address_to: address_to.to_string(),
            value: value,
            block_number,
            date_time: date_time.naive_utc(),
            tx_fee: gas_used * gas_price,
        };

        parsed_transactions.push(parsed_transaction)
    }

    Ok(parsed_transactions)
}

pub fn get_fetch_tx_request_string(
    address: &str,
    start_block: u64,
    end_block: u64,
    page: u32,
) -> String {
    let offset = 1000;
    let sort = "asc";

    ETHERSCAN_API.to_string()
        + MODULE_ACCOUNT
        + ACTION_TXLIST
        + ADDRESS_PLACEHOLDER
        + address
        + START_BLOCK_PLACEHOLDER
        + &start_block.to_string()
        + END_BLOCK_PLACEHOLDER
        + &end_block.to_string()
        + PAGE_PLACEHOLDER
        + &page.to_string()
        + OFFSET_PLACEHOLDER
        + &offset.to_string()
        + SORT_PLACEHOLDER
        + sort
        + API_KEY_PLACEHOLDER
        + &ETHERSCAN_API_KEY
}

pub async fn fetch_transactions(
    (r_start, r_end): (u64, u64),
    transaction_form_input: &TransactionFormInput,
) -> Result<Vec<transactions::Model>, ServerError> {
    let mut page = 1;

    let request =
        get_fetch_tx_request_string(&transaction_form_input.address, r_start, r_end, page);

    let mut resp = reqwest::get(request).await?.json::<Value>().await?;

    let mut transactions = resp["result"]
        .as_array()
        .ok_or(ServerError::ReqwestParsingError)?;

    let mut parsed_transactions: Vec<transactions::Model> = Vec::new();

    // Parse transactions until there are no more transactions left to fetch
    while transactions.len() != 0 {
        let request =
            get_fetch_tx_request_string(&transaction_form_input.address, r_start, r_end, page);

        resp = reqwest::get(request).await?.json::<Value>().await?;
        transactions = resp["result"]
            .as_array()
            .ok_or(ServerError::ReqwestParsingError)?;

        parsed_transactions.extend(parse_transactions(transactions).await?);

        page += 1;
    }

    Ok(parsed_transactions)
}

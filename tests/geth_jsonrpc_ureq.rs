//! Test the geth RPC client. To run these tests access to a geth node is
//! required.
use std::str::FromStr;

use anyhow::Result;
use conquer_once::Lazy;

use ethereum::geth::jsonrpc_ureq::{Client, Url};
use ethereum::geth::GethClient;
use ethereum::types::{BlockNumber, CallRequest};
use ethereum::{Address, ChainId, Ether, Wei};

// URL of the geth node to test against.
const GETH_URL: &str = "http://localhost:8545/";
// `geth -networkid <CHAIN_ID>`
const CHAIN_ID: u32 = 1337;
// All tests transfer Ether from this address to `RECEIVE`.
const SEND: &str = "0x0000000000000000000000000000000000000001";
// All test receive Ether at this address.
const RECEIVE: &str = "0x0000000000000000000000000000000000000002";
// The initial amount of wei in the send account (decimal string, same as in
// `genesis.json`).
const INITIAL_SEND: &str = "1000000000000000000000";
// The initial amount of wei in the receive account (decimal string, same as in
// `genesis.json`).
const INITIAL_RECEIVE: &str = "100000000000000000000";

static SEND_ADDR: Lazy<Address> =
    Lazy::new(|| Address::from_str(SEND).expect("failed to parse const address string"));
static RECEIVE_ADDR: Lazy<Address> =
    Lazy::new(|| Address::from_str(RECEIVE).expect("failed to parse const address string"));

static INITIAL_SEND_BALANCE: Lazy<Ether> = Lazy::new(|| {
    Ether::from(Wei::try_from_dec_str(INITIAL_SEND).expect("failed to parse const amount string"))
});
static INITIAL_RECEIVE_BALANCE: Lazy<Ether> = Lazy::new(|| {
    Ether::from(
        Wei::try_from_dec_str(INITIAL_RECEIVE).expect("failed to parse const amount string"),
    )
});

fn client() -> Client {
    let url = Url::from_str(GETH_URL).expect("failed to parse url");
    Client::new(url)
}

fn empty_eth_call() -> CallRequest {
    CallRequest {
        from: None,
        to: None,
        gas: None,
        gas_price: None,
        value: None,
        data: None,
    }
}

fn earliest() -> BlockNumber {
    BlockNumber::Earliest
}

fn latest() -> BlockNumber {
    BlockNumber::Latest
}

#[test]
fn can_connect_to_geth_node() -> Result<()> {
    let cli = client();
    let _ = cli.client_version()?;
    Ok(())
}

#[test]
fn connected_to_expected_network() -> Result<()> {
    let cli = client();

    let got = cli.chain_id()?;
    let want = ChainId::from(CHAIN_ID);

    assert_eq!(got, want);
    Ok(())
}

#[test]
fn can_get_initial_balance_of_expected_accounts() -> Result<()> {
    let cli = client();

    let got = cli.get_balance(*SEND_ADDR, earliest())?;
    assert_eq!(got, *INITIAL_SEND_BALANCE);

    let got = cli.get_balance(*RECEIVE_ADDR, earliest())?;
    assert_eq!(got, *INITIAL_RECEIVE_BALANCE);

    Ok(())
}

#[test]
fn can_get_initial_transaction_count_of_expected_accounts() -> Result<()> {
    let cli = client();

    let count = cli.get_transaction_count(*SEND_ADDR, earliest())?;
    assert_eq!(count, 0);

    let count = cli.get_transaction_count(*RECEIVE_ADDR, earliest())?;
    assert_eq!(count, 0);

    Ok(())
}

#[test]
fn can_get_gas_price() -> Result<()> {
    let cli = client();
    let _ = cli.gas_price()?;
    Ok(())
}

#[test]
fn can_estimate_gas() -> Result<()> {
    let cli = client();
    let req = empty_eth_call();
    let _ = cli.gas_limit(req, earliest())?;
    Ok(())
}

// Only one unit test sends transactions, this means we can rely on transaction
// count and balances even though the tests are run in parallel.
#[test]
fn transaction() -> Result<()> {
    let cli = client();

    let send_balance_before = cli.get_balance(*SEND_ADDR, latest())?;
    let receive_balance_before = cli.get_balance(*RECEIVE_ADDR, latest())?;

    let send_txn_count_before = cli.get_transaction_count(*SEND_ADDR, latest())?;
    let receive_txn_count_before = cli.get_transaction_count(*RECEIVE_ADDR, latest())?;

    let (hex, amount) = build_transaction()?;
    let hash = cli.send_raw_transaction(hex)?;

    let receipt = cli.get_transaction_receipt(hash)?;
    assert!(receipt.is_some());

    let send_balance_after = cli.get_balance(*SEND_ADDR, latest())?;
    let receive_balance_after = cli.get_balance(*RECEIVE_ADDR, latest())?;

    let send_txn_count_after = cli.get_transaction_count(*SEND_ADDR, latest())?;
    let receive_txn_count_after = cli.get_transaction_count(*RECEIVE_ADDR, latest())?;

    assert!(send_txn_count_before + 1 == send_txn_count_after);
    assert!(receive_txn_count_before + 1 == receive_txn_count_after);

    // We don't know what the fees will be so just check that the after balance is
    // less than the original after deducting the amount sent.
    let with_amount_deducted = send_balance_before - Ether::from(amount.clone());
    assert!(send_balance_after < with_amount_deducted);

    let with_amount_added = receive_balance_before + Ether::from(amount);
    assert_eq!(receive_balance_after, with_amount_added);

    Ok(())
}

// Create a raw signed Ethereum transaction.
fn build_transaction() -> Result<(String, Wei)> {
    todo!()
}

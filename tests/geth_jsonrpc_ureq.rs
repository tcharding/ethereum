//! Test the geth RPC client. To run these tests access to a geth node is
//! required.
use std::str::FromStr;

use anyhow::Result;
use conquer_once::Lazy;

use ethereum::geth::jsonrpc_ureq::{Client, Url};
use ethereum::geth::DefaultBlock;
use ethereum::{Address, Amount, ChainId};

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

static INITIAL_SEND_BALANCE: Lazy<Amount> = Lazy::new(|| {
    Amount::from_wei_dec_str(INITIAL_SEND).expect("failed to parse const amount string")
});
static INITIAL_RECEIVE_BALANCE: Lazy<Amount> = Lazy::new(|| {
    Amount::from_wei_dec_str(INITIAL_RECEIVE).expect("failed to parse const amount string")
});

fn client() -> Client {
    let url = Url::from_str(GETH_URL).expect("failed to parse url");
    Client::new(url)
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

    let got = cli.get_balance(*SEND_ADDR, DefaultBlock::Earliest)?;
    assert_eq!(got, *INITIAL_SEND_BALANCE);

    let got = cli.get_balance(*RECEIVE_ADDR, DefaultBlock::Earliest)?;
    assert_eq!(got, *INITIAL_RECEIVE_BALANCE);

    Ok(())
}

#[test]
fn can_get_gas_price() -> Result<()> {
    let cli = client();

    let _ = cli.gas_price()?;

    Ok(())
}

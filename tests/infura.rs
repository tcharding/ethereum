//! Test the `api` module against Infura.
use std::str::FromStr;

use anyhow::Result;
use clarity::{Address, PrivateKey, Transaction, Uint256};
use conquer_once::Lazy;

use ethereum::api::{Client, Url};
use ethereum::types::{BlockNumber, CallRequest};
use ethereum::ChainId;

// Set up a project at infura.io (set network to Ropsten).
static PROJECT_ID: &str = env!("INFURA_PROJECT_ID");

// Infura endpoint (must have trailing slash).
const ENDPOINT: &str = "https://ropsten.infura.io/v3/";
// Chain id (also network id).
const CHAIN_ID: u32 = 3; // Ropsten

fn alice() -> Address {
    alice_private_key().to_public_key().unwrap()
}

fn bob() -> Address {
    bob_private_key().to_public_key().unwrap()
}

fn alice_private_key() -> PrivateKey {
    let key_material = "aaaaaaaa6422720fab7fee3f875fc2cb399af859156d8f189b930e485674e472";
    PrivateKey::from_str(&key_material).unwrap()
}

fn bob_private_key() -> PrivateKey {
    let key_material = "bbbbbbbb6422720fab7fee3f875fc2cb399af859156d8f189b930e485674e472";
    PrivateKey::from_str(&key_material).unwrap()
}

static CLIENT: Lazy<Client> = Lazy::new(|| {
    let endpoint = format!("{}{}", ENDPOINT, PROJECT_ID);
    let url = Url::from_str(&endpoint).expect("failed to parse url");
    Client::new(url)
});

fn client() -> Client {
    (*CLIENT).clone()
}

fn latest() -> BlockNumber {
    BlockNumber::Latest
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

#[test]
fn can_connect_to_infura() -> Result<()> {
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
fn can_get_balance() -> Result<()> {
    let cli = client();

    let balance = cli.get_balance(alice(), latest())?;
    println!("Alice's current balance: {}", balance);

    let balance = cli.get_balance(bob(), latest())?;
    println!("Bob's current balance: {}", balance);

    Ok(())
}

#[test]
fn can_get_transaction_count() -> Result<()> {
    let cli = client();

    let count = cli.get_transaction_count(alice(), latest())?;
    println!("Alice's current transaction count: {}", count);

    let count = cli.get_transaction_count(bob(), latest())?;
    println!("Bob's current transaction count:: {}", count);

    Ok(())
}

#[test]
fn can_get_gas_price() -> Result<()> {
    let cli = client();
    let price = cli.gas_price()?;
    println!("Current gas price: {}", price);

    Ok(())
}

#[test]
fn can_estimate_gas() -> Result<()> {
    let cli = client();
    let limit = cli.gas_limit(empty_eth_call(), latest())?;
    println!("Current gas limit: {}", limit);

    Ok(())
}

// Only one unit test sends transactions, this means we can rely on transaction
// count and balances even though the tests are run in parallel.
#[test]
fn can_send_transaction() -> Result<()> {
    // Alice's address
    // https://ropsten.etherscan.io/address/0x0EB44ea45B049fc225Cfdf07883dD89C7FeBd8f0
    // Bob's address
    // https://ropsten.etherscan.io/address/0x29F9022a926F25b6b98642C357AEbdf2BfE39970
    let cli = client();
    let nonce = cli.get_transaction_count(alice(), latest())?;

    let gas_limit = 21_000u32;
    let gas_price = cli.gas_price()?;
    let value = Uint256::from_str("10_000_000_000_000_000")?; // 0.01 ether

    let tx = Transaction {
        nonce: nonce.into(),
        gas_price: gas_price.into(),
        gas_limit: gas_limit.into(),
        to: bob(),
        value,
        data: Vec::new(),
        signature: None, // Not signed. Yet.
    };

    let tx_signed: Transaction = tx.sign(&alice_private_key(), None);
    assert!(tx_signed.is_valid());

    let hash = cli.send_raw_transaction(tx_signed.to_string())?;

    Ok(())
}

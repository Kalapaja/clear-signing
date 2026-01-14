use alloy_core::primitives::{Address, Bytes, U256};
use anyhow::{Context, Result};
use clap::Parser;
use clear_signing::display::Display;
use clear_signing::{clear_call::ClearCallContext, resolver::Message};
use clear_signing_format::{format_clear_call, NativeToken};
use std::fs;
use std::path::PathBuf;

mod provider;
use provider::{ClearSigningProvider, ProviderRegistry};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the folder containing configuration files
    folder: PathBuf,
}

#[derive(serde::Deserialize)]
struct Transaction {
    sender: Address,
    to: Address,
    value: U256,
    data: Bytes,
    /// transaction.json contains specific displays for this tx
    displays: Vec<Display>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let folder = cli.folder;
    let path = PathBuf::from("../../examples");

    // Load JSON files
    let tokenlist_path = path.join(&folder).join("tokenlist.json");
    let contractlist_path = path.join(&folder).join("contractlist.json");
    let displays_path = path.join(&folder).join("displays.json");
    let transactions_path = path.join(&folder).join("transactions.json");
    let contacts_path = path.join(&folder).join("contacts.json");

    let tokens: clear_signing_format::TokenList = serde_json::from_str(
        &fs::read_to_string(&tokenlist_path).context("Failed to read tokenlist.json")?,
    )?;
    let tokens = tokens.tokens;
    let contracts: clear_signing_format::ContractList = serde_json::from_str(
        &fs::read_to_string(&contractlist_path).context("Failed to read contractlist.json")?,
    )?;
    let contracts = contracts.contracts;
    let displays: clear_signing::display::DisplaySpecFile = serde_json::from_str(
        &fs::read_to_string(&displays_path).context("Failed to read displays.json")?,
    )?;
    let displays = displays.displays;
    let contacts: Vec<provider::Contact> = if contacts_path.exists() {
        serde_json::from_str(
            &fs::read_to_string(&contacts_path).context("Failed to read contacts.json")?,
        )?
    } else {
        Vec::new()
    };

    // Construct provider
    // Note: NativeToken is not in the file list provided by user.
    // I'll create a default one or look for one. For now default.
    let native_token = NativeToken {
        name: "Ether".to_string(),
        symbol: "ETH".to_string(),
        decimals: 18,
        logo_uri: None,
    };

    let provider = ClearSigningProvider {
        tokens,
        contracts,
        contacts,
        displays: displays.clone(),
        native_token,
    };

    let registry = ProviderRegistry {
        provider: &provider,
    };

    // Parse transactions
    let txs_str =
        fs::read_to_string(&transactions_path).context("Failed to read transactions.json")?;
    let transactions: Vec<Transaction> = serde_json::from_str(&txs_str)?;

    for (i, tx) in transactions.into_iter().enumerate() {
        println!("--- Transaction #{} ---", i + 1);
        let result = (|| -> Result<()> {
            let message = Message {
                sender: tx.sender,
                to: tx.to,
                value: tx.value,
                data: tx.data,
            };

            // Use displays from transactions.json as the context?
            let context = ClearCallContext::new(tx.displays);

            // Parse ClearCall
            let clear_call = context
                .parse_clear_call(message, &registry, 0)
                .map_err(|e| anyhow::anyhow!(e))?;

            // Format output
            let formatted = format_clear_call(&clear_call, &provider, 0, false, None);

            println!("{}", formatted);
            Ok(())
        })();

        if let Err(e) = result {
            eprintln!("Error: {}\n", e);
        } else {
            println!();
        }
    }

    Ok(())
}

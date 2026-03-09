#![no_std]
extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloy_primitives::hex;
use alloy_primitives::utils::format_units;
use alloy_primitives::{Address, U256};
use clear_signing::{ClearCall, Direction, DisplayField, Label, Labels};
use core::ops::Not;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const CONTAINER_LABELS: &str = "labels";

/// ERC-20 token metadata
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Token {
    /// The chain ID of the network where this token is deployed
    pub chain_id: u64,
    /// Token contract address
    pub address: Address,
    /// Optional token ID (e.g., for NFTs)
    pub token_id: Option<U256>,
    /// Token name (e.g., "Wrapped Ether")
    pub name: String,
    /// Token symbol (e.g., "WETH")
    pub symbol: String,
    /// Number of decimals (e.g., 18)
    pub decimals: u8,
    /// Optional logo URI
    #[cfg_attr(feature = "serde", serde(rename = "logoURI"))]
    pub logo_uri: Option<String>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenList {
    #[cfg_attr(
        feature = "serde",
        serde(rename = "$schema", skip_serializing_if = "Option::is_none")
    )]
    pub schema: Option<String>,
    pub name: String,
    pub timestamp: String,
    pub version: Version,
    pub tokens: Vec<Token>,
}

/// Smart contract metadata
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Contract {
    /// The chain ID of the network where this contract is deployed
    pub chain_id: u64,
    /// Contract address
    pub address: Address,
    /// Contract name (e.g., "Uniswap V2 Router")
    pub name: String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractList {
    #[cfg_attr(
        feature = "serde",
        serde(rename = "$schema", skip_serializing_if = "Option::is_none")
    )]
    pub schema: Option<String>,
    pub name: String,
    pub timestamp: String,
    pub version: Version,
    pub contracts: Vec<Contract>,
}

/// Native token metadata
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeToken {
    /// Token name (e.g., "Ether")
    pub name: String,
    /// Token symbol (e.g., "ETH")
    pub symbol: String,
    /// Number of decimals (e.g., 18)
    pub decimals: u8,
    /// Optional logo URI
    pub logo_uri: Option<String>,
}

pub trait MetadataProvider {
    fn get_token(&self, address: Address, token_id: Option<U256>) -> Option<Token>;
    fn get_contract(&self, address: Address) -> Option<Contract>;
    fn get_native_token(&self) -> NativeToken;
    fn get_address_name(&self, address: Address) -> Option<String>;
}

pub fn format_clear_call(
    clear_call: &ClearCall,
    provider: &impl MetadataProvider,
    level: usize,
    detailed: bool,
    locale: Option<&str>,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    let indent = "  ".repeat(level);

    let resolve = |label: &Label| -> String {
        resolve_label(label, &clear_call.labels, locale)
    };

    let title = resolve(&clear_call.title);
    if !title.is_empty() {
        lines.push(format!("{}==={}===", indent, title));
    }

    if detailed {
        let desc = resolve(&clear_call.description);
        if !desc.is_empty() {
            lines.push(format!("{}{}", indent, desc));
        }
    }

    if clear_call.payable || clear_call.clear.not() {
        lines.push("".to_string());
        if clear_call.clear.not() {
            lines.push(format!(
                "{}{}",
                indent, "--- Local display spec ---"
            ));
        }
        if clear_call.payable {
            lines.push(format!(
                "{}{}",
                indent, "!!! This call sends ETH !!!"
            ));
        }
        lines.push("".to_string());
    }

    for field in &clear_call.fields {
        format_field(
            field,
            provider,
            level,
            &mut lines,
            &clear_call.labels,
            detailed,
            locale,
        );
    }

    lines.join("\n")
}

fn format_field(
    field: &DisplayField,
    provider: &impl MetadataProvider,
    level: usize,
    lines: &mut Vec<String>,
    labels: &[Labels],
    detailed: bool,
    locale: Option<&str>,
) {
    let indent = "  ".repeat(level);

    let resolve = |label: &Label| -> String {
        resolve_label(label, labels, locale)
    };

    match field {
        DisplayField::Match {
            title,
            description,
            values,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            for value in values {
                format_field(value, provider, level, lines, labels, detailed, locale);
            }
        }
        DisplayField::Array {
            title,
            description,
            values,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            for item in values {
                for value in item {
                    format_field(value, provider, level + 1, lines, labels, detailed, locale);
                }
            }
        }
        DisplayField::TokenAmount {
            title,
            description,
            token,
            amount,
            direction,
            token_id,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }

            let dir_prefix = match direction {
                Some(Direction::In) => "<-- ",
                Some(Direction::Out) => "--> ",
                None => "",
            };

            let token_suffix = if let Some(id) = token_id {
                format!(" #{}", id)
            } else {
                "".to_string()
            };

            if *amount
                >= U256::from_be_bytes(hex!(
                    "8000000000000000000000000000000000000000000000000000000000000000"
                ))
            {
                let symbol = provider
                    .get_token(*token, *token_id)
                    .map(|t| t.symbol)
                    .unwrap_or_else(|| "Unknown Token".to_string());
                lines.push(format!("{}  {}Unlimited {}", indent, dir_prefix, symbol));
            } else if let Some(token_meta) = provider.get_token(*token, *token_id) {
                let formatted = format_units(*amount, token_meta.decimals)
                    .unwrap_or_else(|_| amount.to_string());
                let formatted = trim_formatted_amount(formatted);
                lines.push(format!(
                    "{}  {}{} {}{}",
                    indent, dir_prefix, formatted, token_meta.symbol, token_suffix
                ).trim_end().to_string());
            } else {
                lines.push(format!(
                    "{}  {}{} (Unknown Token){}",
                    indent, dir_prefix, amount, token_suffix
                ).trim_end().to_string());
            }
        }
        DisplayField::NativeAmount {
            title,
            description,
            amount,
            direction,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }

            let dir_prefix = match direction {
                Some(Direction::In) => "<-- ",
                Some(Direction::Out) => "--> ",
                None => "",
            };

            if *amount
                >= U256::from_be_bytes(hex!(
                    "8000000000000000000000000000000000000000000000000000000000000000"
                ))
            {
                let native_meta = provider.get_native_token();
                lines.push(format!(
                    "{}  {}Unlimited {}",
                    indent, dir_prefix, native_meta.symbol
                ));
            } else {
                let native_meta = provider.get_native_token();
                let formatted = format_units(*amount, native_meta.decimals)
                    .unwrap_or_else(|_| amount.to_string());
                let formatted = trim_formatted_amount(formatted);
                lines.push(format!(
                    "{}  {}{} {}",
                    indent, dir_prefix, formatted, native_meta.symbol
                ));
            }
        }
        DisplayField::Boolean {
            title,
            description,
            value,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            lines.push(format!("{}  {}", indent, if *value { "Yes" } else { "No" }));
        }
        DisplayField::Address {
            title,
            description,
            value,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            if let Some(name) = provider.get_address_name(*value) {
                lines.push(format!("{}  {}", indent, name));
            } else {
                lines.push(format!("{}  {}", indent, value));
            }
        }
        DisplayField::Token {
            title,
            description,
            token,
            token_id,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            let token_suffix = if let Some(id) = token_id {
                format!(" #{}", id)
            } else {
                "".to_string()
            };

            if let Some(token_meta) = provider.get_token(*token, *token_id) {
                lines.push(format!(
                    "{}  {} ({}){}",
                    indent, token_meta.name, token_meta.symbol, token_suffix
                ));
            } else {
                lines.push(format!("{}  {} {}", indent, token, token_suffix));
            }
        }
        DisplayField::Contract {
            title,
            description,
            contract,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            if let Some(contract_meta) = provider.get_contract(*contract) {
                lines.push(format!("{}  {}", indent, contract_meta.name));
            } else {
                lines.push(format!("{}  {} (Unknown Contract)", indent, contract));
            }
        }
        DisplayField::Call {
            title,
            description,
            call,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            lines.push(format_clear_call(
                call,
                provider,
                level + 1,
                detailed,
                locale,
            ));
        }
        DisplayField::Percentage {
            title,
            description,
            value,
            basis,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }

            let val_f = value.to_string().parse::<f64>().unwrap_or(0.0);
            let basis_f = basis.to_string().parse::<f64>().unwrap_or(1.0);
            let pct = (val_f / basis_f) * 100.0;

            lines.push(format!("{}  {:.2}%", indent, pct));
        }
        DisplayField::Duration {
            title,
            description,
            value,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            let seconds = value.as_secs();
            let h = seconds / 3600;
            let m = (seconds % 3600) / 60;
            let s = seconds % 60;
            let mut parts = Vec::new();
            if h > 0 {
                parts.push(format!("{}h", h));
            }
            if m > 0 || h > 0 {
                parts.push(format!("{}m", m));
            }
            parts.push(format!("{}s", s));
            lines.push(format!("{}  {}", indent, parts.join(" ")));
        }
        DisplayField::Datetime {
            title,
            description,
            value,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            let formatted = format_datetime_utc(value.as_secs());
            lines.push(format!("{}  {}", indent, formatted));
        }
        DisplayField::Bitmask {
            title,
            description,
            values,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            let resolved_values: Vec<String> = values.iter().map(resolve).collect();
            lines.push(format!("{}  [{}]", indent, resolved_values.join(", ")));
        }
        DisplayField::String {
            title,
            description,
            value,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            lines.push(format!("{}  {}", indent, value));
        }
        DisplayField::Bytes {
            title,
            description,
            value,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            lines.push(format!("{}  0x{}", indent, hex::encode(value)));
        }
        DisplayField::Int {
            title,
            description,
            value,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            lines.push(format!("{}  {}", indent, value));
        }
        DisplayField::Uint {
            title,
            description,
            value,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            lines.push(format!("{}  {}", indent, value));
        }
        DisplayField::Units {
            title,
            description,
            value,
            decimals,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            let decimals: usize = decimals.try_into().unwrap_or(0);
            let s = value.to_string();
            let formatted = if decimals == 0 {
                s
            } else if s.len() <= decimals {
                let mut padded = "0.".to_string();
                for _ in 0..(decimals - s.len()) {
                    padded.push('0');
                }
                padded.push_str(&s);
                trim_formatted_amount(padded)
            } else {
                let (integer, fractional) = s.split_at(s.len() - decimals);
                trim_formatted_amount(format!("{}.{}", integer, fractional))
            };
            lines.push(format!("{}  {}", indent, formatted));
        }
        DisplayField::Switch {
            title,
            description,
            fields,
        } => {
            let title = resolve(title);
            if !title.is_empty() {
                lines.push(format_title(&title, &indent));
            }
            if detailed {
                let desc = resolve(description);
                if !desc.is_empty() {
                    lines.push(format!("{}{}", indent, desc));
                }
            }
            for field in fields {
                format_field(field, provider, level, lines, labels, detailed, locale);
            }
        }
    }
}

fn format_title(title: &str, indent: &str) -> String {
    format!("{}{}", indent, title)
}

fn trim_formatted_amount(s: String) -> String {
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0');
        if let Some(stripped) = trimmed.strip_suffix('.') {
            stripped.to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        s
    }
}

fn format_datetime_utc(timestamp: u64) -> String {
    // Convert Unix timestamp to UTC date/time components
    const SECONDS_PER_DAY: u64 = 86400;
    const SECONDS_PER_HOUR: u64 = 3600;
    const SECONDS_PER_MINUTE: u64 = 60;

    let days_since_epoch = timestamp / SECONDS_PER_DAY;
    let seconds_today = timestamp % SECONDS_PER_DAY;

    let hour = seconds_today / SECONDS_PER_HOUR;
    let minute = (seconds_today % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;

    // Calculate year, month, day from days since epoch (1970-01-01)
    let mut year = 1970;
    let mut days_remaining = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days_remaining < days_in_year {
            break;
        }
        days_remaining -= days_in_year;
        year += 1;
    }

    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &days_in_month in &days_in_months {
        if days_remaining < days_in_month {
            break;
        }
        days_remaining -= days_in_month;
        month += 1;
    }

    let day = days_remaining + 1;

    format!("{:04}-{:02}-{:02} {:02}:{:02} UTC", year, month, day, hour, minute)
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn resolve_label(label: &str, labels: &[Labels], locale: Option<&str>) -> String {
    let prefix = format!("${}.", CONTAINER_LABELS);
    let key = match label.strip_prefix(prefix.as_str()) {
        Some(key) if !key.is_empty() => key,
        _ => return label.to_string(),
    };

    let target_labels = if let Some(locale) = locale {
        labels.iter().find(|entry| entry.locale == locale)
    } else {
        labels.first()
    };

    if let Some(target) = target_labels
        && let Some(entry) = target.items.iter().find(|entry| entry.key == key)
    {
        return entry.value.clone();
    }

    label.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloy_primitives::{address, U256};

    const TOKEN_ADDR: Address = address!("0000000000000000000000000000000000000111");
    const CONTRACT_ADDR: Address = address!("0000000000000000000000000000000000000222");
    const RECIPIENT_ADDR: Address = address!("0000000000000000000000000000000000000333");

    struct MockMetadataProvider;

    impl MetadataProvider for MockMetadataProvider {
        fn get_token(&self, address: Address, token_id: Option<U256>) -> Option<Token> {
            if address == TOKEN_ADDR {
                Some(Token {
                    chain_id: 1,
                    address: TOKEN_ADDR,
                    name: "Test Token".to_string(),
                    symbol: "TST".to_string(),
                    decimals: 18,
                    logo_uri: None,
                    token_id,
                })
            } else {
                None
            }
        }

        fn get_contract(&self, address: Address) -> Option<Contract> {
            if address == CONTRACT_ADDR {
                Some(Contract {
                    chain_id: 1,
                    address: CONTRACT_ADDR,
                    name: "Test Contract".to_string(),
                })
            } else {
                None
            }
        }

        fn get_native_token(&self) -> NativeToken {
            NativeToken {
                name: "Ether".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                logo_uri: None,
            }
        }

        fn get_address_name(&self, address: Address) -> Option<String> {
            if address == RECIPIENT_ADDR {
                Some("Recipient Name".to_string())
            } else if address == TOKEN_ADDR {
                Some("Test Token".to_string())
            } else if address == CONTRACT_ADDR {
                Some("Test Contract".to_string())
            } else {
                None
            }
        }
    }

    #[test]
    fn test_format_clear_call() {
        use clear_signing::{ClearCall, DisplayField};
        use core::time::Duration;

        let provider = MockMetadataProvider;

        // Construct a sample ClearCall with ALL variants
        let clear_call = ClearCall {
            title: "Swap Tokens".to_string(),
            description: "Swapping ETH for DAI".to_string(),
            payable: true,
            clear: true,
            labels: vec![],
            fields: vec![
                DisplayField::NativeAmount {
                    title: "Amount In".to_string(),
                    description: "Amount of ETH to swap".to_string(),
                    amount: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
                    direction: None,
                },
                DisplayField::TokenAmount {
                    title: "Amount Out".to_string(),
                    description: "Expected amount of DAI".to_string(),
                    token: TOKEN_ADDR,
                    amount: U256::from(2000_000000000000000000u128), // 2000 DAI
                    direction: Some(Direction::Out),
                    token_id: None,
                },
                DisplayField::TokenAmount {
                    title: "Amount No Dir".to_string(),
                    description: "".to_string(),
                    token: TOKEN_ADDR,
                    amount: U256::from(1000_000000000000000000u128), // 1000 DAI
                    direction: None,
                    token_id: None,
                },
                DisplayField::Token {
                    title: "Token Out".to_string(),
                    description: "".to_string(),
                    token: TOKEN_ADDR,
                    token_id: None,
                },
                DisplayField::TokenAmount {
                    title: "NFT Transfer".to_string(),
                    description: "".to_string(),
                    token: TOKEN_ADDR,
                    amount: U256::from(1000000000000000000u64), // 1 TST
                    direction: Some(Direction::Out),
                    token_id: Some(U256::from(42)),
                },
                DisplayField::Contract {
                    title: "Router".to_string(),
                    description: "".to_string(),
                    contract: CONTRACT_ADDR,
                },
                DisplayField::Boolean {
                    title: "Use Wrappers".to_string(),
                    description: "".to_string(),
                    value: true,
                },
                DisplayField::Percentage {
                    title: "Slippage".to_string(),
                    description: "".to_string(),
                    value: U256::from(50),
                    basis: U256::from(10000),
                },
                DisplayField::Duration {
                    title: "Timeout".to_string(),
                    description: "".to_string(),
                    value: Duration::from_secs(3665), // 1h 1m 5s
                },
                DisplayField::Datetime {
                    title: "Deadline".to_string(),
                    description: "".to_string(),
                    value: Duration::from_secs(1736328584),
                },
                DisplayField::Array {
                    title: "Array".to_string(),
                    description: "Array description".to_string(),
                    values: vec![
                        vec![DisplayField::String {
                            title: "Item 1".to_string(),
                            description: "".to_string(),
                            value: "Value 1".to_string(),
                        }],
                        vec![DisplayField::String {
                            title: "Item 2".to_string(),
                            description: "".to_string(),
                            value: "Value 2".to_string(),
                        }],
                    ],
                },
                DisplayField::Bitmask {
                    title: "Options".to_string(),
                    description: "".to_string(),
                    values: vec![
                        "Permit".to_string(),
                        "Wrap".to_string(),
                    ],
                },
                DisplayField::String {
                    title: "Memo".to_string(),
                    description: "".to_string(),
                    value: "Hello Gemini".to_string(),
                },
                DisplayField::Bytes {
                    title: "Extra Data".to_string(),
                    description: "".to_string(),
                    value: vec![0xde, 0xad, 0xbe, 0xef].into(),
                },
                DisplayField::Int {
                    title: "Profit/Loss".to_string(),
                    description: "".to_string(),
                    value: (-100_i128).try_into().unwrap(),
                },
                DisplayField::Uint {
                    title: "Nonce".to_string(),
                    description: "".to_string(),
                    value: U256::from(42),
                },
                DisplayField::Address {
                    title: "Recipient".to_string(),
                    description: "".to_string(),
                    value: RECIPIENT_ADDR,
                },
                DisplayField::Call {
                    title: "Sub-call".to_string(),
                    description: "Inner transaction details".to_string(),
                    call: ClearCall {
                        clear: false,
                        title: "Approval".to_string(),
                        description: "Approve DAI for router".to_string(),
                        payable: false,
                        labels: vec![],
                        fields: vec![DisplayField::TokenAmount {
                            title: "Allowance".to_string(),
                            description: "".to_string(),
                            token: TOKEN_ADDR,
                            amount: U256::MAX,
                            direction: Some(Direction::In),
                            token_id: None,
                        }],
                    },
                },
                DisplayField::Match {
                    title: "Match Group".to_string(),
                    description: "Group description".to_string(),
                    values: vec![
                        DisplayField::String {
                            title: "Inner Field 1".to_string(),
                            description: "".to_string(),
                            value: "Inner Value 1".to_string(),
                        },
                        DisplayField::String {
                            title: "Inner Field 2".to_string(),
                            description: "".to_string(),
                            value: "Inner Value 2".to_string(),
                        },
                    ],
                },
                DisplayField::Units {
                    title: "Price".to_string(),
                    description: "Price in USD".to_string(),
                    value: U256::from(1234567),
                    decimals: U256::from(6),
                },
            ],
        };

        let formatted = format_clear_call(&clear_call, &provider, 0, true, None);
        let expected_lines = vec![
            "===Swap Tokens===",
            "Swapping ETH for DAI",
            "",
            "!!! This call sends ETH !!!",
            "",
            "Amount In",
            "Amount of ETH to swap",
            "  1 ETH",
            "Amount Out",
            "Expected amount of DAI",
            "  --> 2000 TST",
            "Amount No Dir",
            "  1000 TST",
            "Token Out",
            "  Test Token (TST)",
            "NFT Transfer",
            "  --> 1 TST #42",
            "Router",
            "  Test Contract",
            "Use Wrappers",
            "  Yes",
            "Slippage",
            "  0.50%",
            "Timeout",
            "  1h 1m 5s",
            "Deadline",
            "  2025-01-08 09:29 UTC",
            "Array",
            "Array description",
            "  Item 1",
            "    Value 1",
            "  Item 2",
            "    Value 2",
            "Options",
            "  [Permit, Wrap]",
            "Memo",
            "  Hello Gemini",
            "Extra Data",
            "  0xdeadbeef",
            "Profit/Loss",
            "  -100",
            "Nonce",
            "  42",
            "Recipient",
            "  Recipient Name",
            "Sub-call",
            "Inner transaction details",
            "  ===Approval===",
            "  Approve DAI for router",
            "",
            "  --- Local display spec ---",
            "",
            "  Allowance",
            "    <-- Unlimited TST",
            "Match Group",
            "Group description",
            "Inner Field 1",
            "  Inner Value 1",
            "Inner Field 2",
            "  Inner Value 2",
            "Price",
            "Price in USD",
            "  1.234567",
        ];
        let expected = expected_lines.join("\n");

        assert_eq!(formatted, expected);
    }
}

#![no_std]
extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloy_core::hex;
use alloy_core::primitives::utils::format_units;
use alloy_core::primitives::{Address, U256};
use clear_signing::fields::{ClearCall, Direction, DisplayField, Label};
use clear_signing::reference::Reference;
use serde::{Deserialize, Serialize};

/// ERC-20 token metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// The chain ID of the network where this token is deployed
    pub chain_id: u64,
    /// Token contract address
    pub address: Address,
    /// Token name (e.g., "Wrapped Ether")
    pub name: String,
    /// Token symbol (e.g., "WETH")
    pub symbol: String,
    /// Number of decimals (e.g., 18)
    pub decimals: u8,
    /// Optional logo URI
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenList {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub name: String,
    pub timestamp: String,
    pub version: Version,
    pub tokens: Vec<Token>,
}

/// Smart contract metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    /// The chain ID of the network where this contract is deployed
    pub chain_id: u64,
    /// Contract address
    pub address: Address,
    /// Contract name (e.g., "Uniswap V2 Router")
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractList {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub name: String,
    pub timestamp: String,
    pub version: Version,
    pub contracts: Vec<Contract>,
}

/// Native token metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    fn get_token(&self, address: Address) -> Option<Token>;
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
        label
            .resolve(&clear_call.labels, locale)
            .unwrap_or_else(|_| match label {
                Reference::Literal(s) => s.clone(),
                Reference::Identifier {
                    identifier: _,
                    reference,
                } => reference.clone(),
            })
    };

    if level == 0 {
        lines.push("---------------------------------------------------".to_string());
    }

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

    if clear_call.payable {
        lines.push("".to_string());
        lines.push(format!(
            "{}{}",
            indent, "!!! This transaction will send ETH !!!"
        ));
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

    if level == 0 {
        lines.push("---------------------------------------------------".to_string());
    }

    lines.join("\n")
}

fn format_title(title: &str, indent: &str) -> String {
    format!("{}{}", indent, title)
}

fn format_field(
    field: &DisplayField,
    provider: &impl MetadataProvider,
    level: usize,
    lines: &mut Vec<String>,
    labels: &[clear_signing::display::Labels],
    detailed: bool,
    locale: Option<&str>,
) {
    let indent = "  ".repeat(level);

    let resolve = |label: &Label| -> String {
        label
            .resolve(labels, locale)
            .unwrap_or_else(|_| match label {
                Reference::Literal(s) => s.clone(),
                Reference::Identifier {
                    identifier: _,
                    reference,
                } => reference.clone(),
            })
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
                let symbol = provider
                    .get_token(*token)
                    .map(|t| t.symbol)
                    .unwrap_or_else(|| "Unknown Token".to_string());
                lines.push(format!("{}  {}Unlimited {}", indent, dir_prefix, symbol));
            } else if let Some(token_meta) = provider.get_token(*token) {
                let formatted = format_units(*amount, token_meta.decimals)
                    .unwrap_or_else(|_| amount.to_string());
                let formatted = trim_formatted_amount(formatted);
                lines.push(format!(
                    "{}  {}{} {}",
                    indent, dir_prefix, formatted, token_meta.symbol
                ));
            } else {
                lines.push(format!(
                    "{}  {}{} (Unknown Token)",
                    indent, dir_prefix, amount
                ));
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
                lines.push(format!("{}  {} ({})", indent, name, value));
            } else {
                lines.push(format!("{}  {}", indent, value));
            }
        }
        DisplayField::Token {
            title,
            description,
            token,
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
            if let Some(token_meta) = provider.get_token(*token) {
                lines.push(format!(
                    "{}  {} ({})",
                    indent, token_meta.name, token_meta.symbol
                ));
            } else {
                lines.push(format!("{}  {} (Unknown Token)", indent, token));
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
            use chrono::TimeZone;
            let dt = chrono::Utc
                .timestamp_opt(value.as_secs() as i64, 0)
                .single()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| format!("{}s", value.as_secs()));
            lines.push(format!("{}  {}", indent, dt));
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
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloy_core::primitives::{U256, address};

    const TOKEN_ADDR: Address = address!("0000000000000000000000000000000000000111");
    const CONTRACT_ADDR: Address = address!("0000000000000000000000000000000000000222");
    const RECIPIENT_ADDR: Address = address!("0000000000000000000000000000000000000333");

    struct MockMetadataProvider;

    impl MetadataProvider for MockMetadataProvider {
        fn get_token(&self, address: Address) -> Option<Token> {
            if address == TOKEN_ADDR {
                Some(Token {
                    chain_id: 1,
                    address: TOKEN_ADDR,
                    name: "Test Token".to_string(),
                    symbol: "TST".to_string(),
                    decimals: 18,
                    logo_uri: None,
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
        use clear_signing::fields::{ClearCall, DisplayField};
        use core::time::Duration;

        let provider = MockMetadataProvider;

        // Construct a sample ClearCall with ALL variants
        let clear_call = ClearCall {
            title: Reference::Literal("Swap Tokens".to_string()),
            description: Reference::Literal("Swapping ETH for DAI".to_string()),
            payable: true,
            clear: true,
            labels: vec![],
            fields: vec![
                DisplayField::NativeAmount {
                    title: Reference::Literal("Amount In".to_string()),
                    description: Reference::Literal("Amount of ETH to swap".to_string()),
                    amount: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
                    direction: None,
                },
                DisplayField::TokenAmount {
                    title: Reference::Literal("Amount Out".to_string()),
                    description: Reference::Literal("Expected amount of DAI".to_string()),
                    token: TOKEN_ADDR,
                    amount: U256::from(2000_000000000000000000u128), // 2000 DAI
                    direction: Some(Direction::Out),
                },
                DisplayField::TokenAmount {
                    title: Reference::Literal("Amount No Dir".to_string()),
                    description: Reference::Literal("".to_string()),
                    token: TOKEN_ADDR,
                    amount: U256::from(1000_000000000000000000u128), // 1000 DAI
                    direction: None,
                },
                DisplayField::Token {
                    title: Reference::Literal("Token Out".to_string()),
                    description: Reference::Literal("".to_string()),
                    token: TOKEN_ADDR,
                },
                DisplayField::Contract {
                    title: Reference::Literal("Router".to_string()),
                    description: Reference::Literal("".to_string()),
                    contract: CONTRACT_ADDR,
                },
                DisplayField::Boolean {
                    title: Reference::Literal("Use Wrappers".to_string()),
                    description: Reference::Literal("".to_string()),
                    value: true,
                },
                DisplayField::Percentage {
                    title: Reference::Literal("Slippage".to_string()),
                    description: Reference::Literal("".to_string()),
                    value: U256::from(50),
                    basis: U256::from(10000),
                },
                DisplayField::Duration {
                    title: Reference::Literal("Timeout".to_string()),
                    description: Reference::Literal("".to_string()),
                    value: Duration::from_secs(3665), // 1h 1m 5s
                },
                DisplayField::Datetime {
                    title: Reference::Literal("Deadline".to_string()),
                    description: Reference::Literal("".to_string()),
                    value: Duration::from_secs(1736328584),
                },
                DisplayField::Array {
                    title: Reference::Literal("Array".to_string()),
                    description: Reference::Literal("Array description".to_string()),
                    values: vec![
                        vec![DisplayField::String {
                            title: Reference::Literal("Item 1".to_string()),
                            description: Reference::Literal("".to_string()),
                            value: "Value 1".to_string(),
                        }],
                        vec![DisplayField::String {
                            title: Reference::Literal("Item 2".to_string()),
                            description: Reference::Literal("".to_string()),
                            value: "Value 2".to_string(),
                        }],
                    ],
                },
                DisplayField::Bitmask {
                    title: Reference::Literal("Options".to_string()),
                    description: Reference::Literal("".to_string()),
                    values: vec![
                        Reference::Literal("Permit".to_string()),
                        Reference::Literal("Wrap".to_string()),
                    ],
                },
                DisplayField::String {
                    title: Reference::Literal("Memo".to_string()),
                    description: Reference::Literal("".to_string()),
                    value: "Hello Gemini".to_string(),
                },
                DisplayField::Bytes {
                    title: Reference::Literal("Extra Data".to_string()),
                    description: Reference::Literal("".to_string()),
                    value: vec![0xde, 0xad, 0xbe, 0xef].into(),
                },
                DisplayField::Int {
                    title: Reference::Literal("Profit/Loss".to_string()),
                    description: Reference::Literal("".to_string()),
                    value: (-100_i128).try_into().unwrap(),
                },
                DisplayField::Uint {
                    title: Reference::Literal("Nonce".to_string()),
                    description: Reference::Literal("".to_string()),
                    value: U256::from(42),
                },
                DisplayField::Address {
                    title: Reference::Literal("Recipient".to_string()),
                    description: Reference::Literal("".to_string()),
                    value: RECIPIENT_ADDR,
                },
                DisplayField::Call {
                    title: Reference::Literal("Sub-call".to_string()),
                    description: Reference::Literal("Inner transaction details".to_string()),
                    call: ClearCall {
                        clear: false,
                        title: Reference::Literal("Approval".to_string()),
                        description: Reference::Literal("Approve DAI for router".to_string()),
                        payable: false,
                        labels: vec![],
                        fields: vec![DisplayField::TokenAmount {
                            title: Reference::Literal("Allowance".to_string()),
                            description: Reference::Literal("".to_string()),
                            token: TOKEN_ADDR,
                            amount: U256::MAX,
                            direction: Some(Direction::In),
                        }],
                    },
                },
                DisplayField::Match {
                    title: Reference::Literal("Match Group".to_string()),
                    description: Reference::Literal("Group description".to_string()),
                    values: vec![
                        DisplayField::String {
                            title: Reference::Literal("Inner Field 1".to_string()),
                            description: Reference::Literal("".to_string()),
                            value: "Inner Value 1".to_string(),
                        },
                        DisplayField::String {
                            title: Reference::Literal("Inner Field 2".to_string()),
                            description: Reference::Literal("".to_string()),
                            value: "Inner Value 2".to_string(),
                        },
                    ],
                },
            ],
        };

        let formatted = format_clear_call(&clear_call, &provider, 0, true, None);
        let expected_lines = vec![
            "---------------------------------------------------",
            "===Swap Tokens===",
            "Swapping ETH for DAI",
            "",
            "!!! This transaction will send ETH !!!",
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
            "Router",
            "  Test Contract",
            "Use Wrappers",
            "  Yes",
            "Slippage",
            "  0.50%",
            "Timeout",
            "  1h 1m 5s",
            "Deadline",
            "  2025-01-08 09:29:44 UTC",
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
            "  Recipient Name (0x0000000000000000000000000000000000000333)",
            "Sub-call",
            "Inner transaction details",
            "  ===Approval===",
            "  Approve DAI for router",
            "  Allowance",
            "    <-- Unlimited TST",
            "Match Group",
            "Group description",
            "Inner Field 1",
            "  Inner Value 1",
            "Inner Field 2",
            "  Inner Value 2",
            "---------------------------------------------------",
        ];
        let expected = expected_lines.join("\n");

        assert_eq!(formatted, expected);
    }
}

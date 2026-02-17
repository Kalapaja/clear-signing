use crate::error::ParseError;
use alloy_primitives::{keccak256, Address, B256};
use alloc::string::String;
use alloc::vec::Vec;
use core::clone::Clone;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct DisplaySpecFile {
    pub displays: Vec<Display>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Display {
    pub address: Address,
    pub abi: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub title: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub description: String,
    pub fields: Vec<Field>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub labels: Vec<Labels>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    #[cfg_attr(feature = "serde", serde(default))]
    pub title: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub description: String,
    pub format: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub checks: Vec<Vec<Check>>,
    pub params: Vec<Entry>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub fields: Vec<Field>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Labels {
    pub locale: String,
    pub items: Vec<Entry>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: String,
    pub value: String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub left: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub op: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub right: String,
}

impl Display {
    pub fn validate(&self) -> bool {
        self.abi.len() < 1024
            && self.fields.len() < 256
            && self.labels.len() < 256
            && self
                .fields
                .iter()
                .all(|field| field.format.len() < 256 && field.params.len() < 256)
            && self.labels.iter().all(|labels| labels.items.len() < 256)
    }

    pub fn hash_struct(&self) -> B256 {
        eip712_hash_display(self)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Format {
    TokenAmount,
    NativeAmount,
    Contract,
    Token,
    Address,
    Bytes,
    String,
    Call,
    Boolean,
    Int,
    Uint,
    Percentage,
    Duration,
    Datetime,
    Bitmask,
    Match,
    Array,
}

impl Format {
    pub fn from(format: &str) -> Result<Self, ParseError> {
        match format {
            "tokenAmount" => Ok(Format::TokenAmount),
            "nativeAmount" => Ok(Format::NativeAmount),
            "contract" => Ok(Format::Contract),
            "token" => Ok(Format::Token),
            "address" => Ok(Format::Address),
            "bytes" => Ok(Format::Bytes),
            "string" => Ok(Format::String),
            "call" => Ok(Format::Call),
            "boolean" => Ok(Format::Boolean),
            "int" => Ok(Format::Int),
            "uint" => Ok(Format::Uint),
            "percentage" => Ok(Format::Percentage),
            "duration" => Ok(Format::Duration),
            "datetime" => Ok(Format::Datetime),
            "bitmask" => Ok(Format::Bitmask),
            "match" => Ok(Format::Match),
            "array" => Ok(Format::Array),
            _ => Err(ParseError::UnknownFormat(format.into())),
        }
    }
}

// EIP-712 type hashes (lazily computed)
fn check_typehash() -> B256 {
    keccak256("Check(string left,string op,string right)")
}

fn entry_typehash() -> B256 {
    keccak256("Entry(string key,string value)")
}

fn field_typehash() -> B256 {
    keccak256("Field(string title,string description,string format,Check[][] checks,Entry[] params,Field[] fields)Check(string left,string op,string right)Entry(string key,string value)")
}

fn labels_typehash() -> B256 {
    keccak256("Labels(string locale,Entry[] items)Entry(string key,string value)")
}

fn display_typehash() -> B256 {
    keccak256("Display(address address,string abi,string title,string description,Field[] fields,Labels[] labels)Check(string left,string op,string right)Entry(string key,string value)Field(string title,string description,string format,Check[][] checks,Entry[] params,Field[] fields)Labels(string locale,Entry[] items)")
}

// Manual EIP-712 encoding functions for recursive types
// These match Solidity's abi.encode() behavior
fn eip712_hash_check(check: &Check) -> B256 {
    use alloy_sol_types::SolValue;

    let left_hash = keccak256(check.left.as_bytes());
    let op_hash = keccak256(check.op.as_bytes());
    let right_hash = keccak256(check.right.as_bytes());

    // Use Solidity's abi.encode which pads each element to 32 bytes
    let encoded = (check_typehash(), left_hash, op_hash, right_hash).abi_encode();
    keccak256(&encoded)
}

fn eip712_hash_entry(entry: &Entry) -> B256 {
    use alloy_sol_types::SolValue;

    let key_hash = keccak256(entry.key.as_bytes());
    let value_hash = keccak256(entry.value.as_bytes());

    // Use Solidity's abi.encode which pads each element to 32 bytes
    let encoded = (entry_typehash(), key_hash, value_hash).abi_encode();
    keccak256(&encoded)
}

fn eip712_hash_field(field: &Field) -> B256 {
    use alloy_sol_types::SolValue;

    // Hash checks array (array of arrays)
    // Each check is hashed, then each check group, then the whole array
    let checks_hashes: Vec<B256> = field.checks.iter().map(|check_group| {
        let inner_hashes: Vec<B256> = check_group.iter()
            .map(|check| eip712_hash_check(check))
            .collect();
        // Concatenate inner hashes and hash them (ABI encoding of bytes32[])
        let inner_bytes: Vec<u8> = inner_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
        keccak256(&inner_bytes)
    }).collect();
    // Concatenate outer hashes and hash them
    let checks_bytes: Vec<u8> = checks_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
    let checks_hash = keccak256(&checks_bytes);

    // Hash params array
    let params_hashes: Vec<B256> = field.params.iter().map(|e| eip712_hash_entry(e)).collect();
    let params_bytes: Vec<u8> = params_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
    let params_hash = keccak256(&params_bytes);

    // Hash fields array (recursive)
    let fields_hashes: Vec<B256> = field.fields.iter().map(|f| eip712_hash_field(f)).collect();
    let fields_bytes: Vec<u8> = fields_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
    let fields_hash = keccak256(&fields_bytes);

    let title_hash = keccak256(field.title.as_bytes());
    let description_hash = keccak256(field.description.as_bytes());
    let format_hash = keccak256(field.format.as_bytes());

    // Use Solidity's abi.encode which pads each element to 32 bytes
    let encoded = (
        field_typehash(),
        title_hash,
        description_hash,
        format_hash,
        checks_hash,
        params_hash,
        fields_hash,
    ).abi_encode();
    keccak256(&encoded)
}

fn eip712_hash_labels(labels: &Labels) -> B256 {
    use alloy_sol_types::SolValue;

    let items_hashes: Vec<B256> = labels.items.iter().map(|e| eip712_hash_entry(e)).collect();
    let items_bytes: Vec<u8> = items_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
    let items_hash = keccak256(&items_bytes);
    let locale_hash = keccak256(labels.locale.as_bytes());

    // Use Solidity's abi.encode which pads each element to 32 bytes
    let encoded = (labels_typehash(), locale_hash, items_hash).abi_encode();
    keccak256(&encoded)
}

fn eip712_hash_display(display: &Display) -> B256 {
    use alloy_sol_types::SolValue;

    let fields_hashes: Vec<B256> = display.fields.iter().map(|f| eip712_hash_field(f)).collect();
    let fields_bytes: Vec<u8> = fields_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
    let fields_hash = keccak256(&fields_bytes);

    let labels_hashes: Vec<B256> = display.labels.iter().map(|l| eip712_hash_labels(l)).collect();
    let labels_bytes: Vec<u8> = labels_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
    let labels_hash = keccak256(&labels_bytes);

    let abi_hash = keccak256(display.abi.as_bytes());
    let title_hash = keccak256(display.title.as_bytes());
    let description_hash = keccak256(display.description.as_bytes());

    // Use Solidity's abi.encode which pads each element to 32 bytes
    let encoded = (
        display_typehash(),
        display.address,
        abi_hash,
        title_hash,
        description_hash,
        fields_hash,
        labels_hash,
    ).abi_encode();
    keccak256(&encoded)
}

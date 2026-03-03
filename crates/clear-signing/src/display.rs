use alloy_primitives::{keccak256, B256};
use alloc::string::String;
use alloc::vec::Vec;
use core::clone::Clone;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const MAX_STRING_LEN: usize = 256;
const MAX_ARRAY_LEN: usize = 256;
const MAX_ABI_LEN: usize = 1024;
const MAX_FIELD_DEPTH: usize = 16;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Display {
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
    pub case: Vec<String>,
    pub params: Vec<Entry>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub fields: Vec<Field>,
}

impl Field {
    pub fn validate(&self, depth: usize) -> bool {
        if depth >= MAX_FIELD_DEPTH {
            return false;
        }

        if self.title.len() >= MAX_STRING_LEN
            || self.description.len() >= MAX_STRING_LEN
            || self.format.len() >= MAX_STRING_LEN
        {
            return false;
        }

        if self.case.len() >= MAX_ARRAY_LEN
            || self.params.len() >= MAX_ARRAY_LEN
            || self.fields.len() >= MAX_ARRAY_LEN
        {
            return false;
        }

        if !self.case.iter().all(|c| c.len() < MAX_STRING_LEN) {
            return false;
        }

        if !self.params.iter().all(|entry| entry.validate()) {
            return false;
        }

        self.fields
            .iter()
            .all(|field| field.validate(depth + 1))
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Labels {
    pub locale: String,
    pub items: Vec<Entry>,
}

impl Labels {
    pub fn validate(&self) -> bool {
        self.locale.len() < MAX_STRING_LEN
            && self.items.len() < MAX_ARRAY_LEN
            && self.items.iter().all(|entry| entry.validate())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: String,
    pub value: String,
}

impl Entry {
    pub fn validate(&self) -> bool {
        self.key.len() < MAX_STRING_LEN && self.value.len() < MAX_STRING_LEN
    }
}

impl Display {
    pub fn validate(&self) -> bool {
        self.abi.len() < MAX_ABI_LEN
            && self.fields.len() < MAX_ARRAY_LEN
            && self.labels.len() < MAX_ARRAY_LEN
            && self.fields.iter().all(|field| field.validate(0))
            && self.labels.iter().all(|labels| labels.validate())
    }

    pub fn hash_struct(&self) -> B256 {
        eip712_hash_display(self)
    }
}


// EIP-712 type hashes (lazily computed)
fn entry_typehash() -> B256 {
    keccak256("Entry(string key,string value)")
}

fn field_typehash() -> B256 {
    keccak256("Field(string title,string description,string format,string[] case,Entry[] params,Field[] fields)Entry(string key,string value)")
}

fn labels_typehash() -> B256 {
    keccak256("Labels(string locale,Entry[] items)Entry(string key,string value)")
}

fn display_typehash() -> B256 {
    keccak256("Display(string abi,string title,string description,Field[] fields,Labels[] labels)Entry(string key,string value)Field(string title,string description,string format,string[] case,Entry[] params,Field[] fields)Labels(string locale,Entry[] items)")
}

fn eip712_hash_entry(entry: &Entry) -> B256 {
    use alloy_sol_types::SolValue;

    let key_hash = keccak256(entry.key.as_bytes());
    let value_hash = keccak256(entry.value.as_bytes());

    let encoded = (entry_typehash(), key_hash, value_hash).abi_encode();
    keccak256(&encoded)
}

fn eip712_hash_field(field: &Field) -> B256 {
    use alloy_sol_types::SolValue;

    let case_hashes: Vec<B256> = field.case.iter()
        .map(|c| keccak256(c.as_bytes()))
        .collect();
    let case_bytes: Vec<u8> = case_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
    let case_hash = keccak256(&case_bytes);

    let params_hashes: Vec<B256> = field.params.iter().map(|e| eip712_hash_entry(e)).collect();
    let params_bytes: Vec<u8> = params_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
    let params_hash = keccak256(&params_bytes);

    let fields_hashes: Vec<B256> = field.fields.iter().map(|f| eip712_hash_field(f)).collect();
    let fields_bytes: Vec<u8> = fields_hashes.iter().flat_map(|h| h.as_slice()).copied().collect();
    let fields_hash = keccak256(&fields_bytes);

    let title_hash = keccak256(field.title.as_bytes());
    let description_hash = keccak256(field.description.as_bytes());
    let format_hash = keccak256(field.format.as_bytes());

    let encoded = (
        field_typehash(),
        title_hash,
        description_hash,
        format_hash,
        case_hash,
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

    let encoded = (
        display_typehash(),
        abi_hash,
        title_hash,
        description_hash,
        fields_hash,
        labels_hash,
    ).abi_encode();
    keccak256(&encoded)
}

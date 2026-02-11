use crate::error::ParseError;
use alloy_primitives::B256;
use alloy_sol_types::sol;
use alloy_sol_types::SolStruct;
use core::clone::Clone;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct DisplaySpecFile {
    pub displays: alloc::vec::Vec<Display>,
}

sol! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Debug, PartialEq)]
    struct Display {
        address address;
        string abi;
        #[cfg_attr(feature = "serde", serde(default))]
        string title;
        #[cfg_attr(feature = "serde", serde(default))]
        string description;
        Field[] fields;
        #[cfg_attr(feature = "serde", serde(default))]
        Labels[] labels;
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Debug, PartialEq)]
    struct Field {
        #[cfg_attr(feature = "serde", serde(default))]
        string title;
        #[cfg_attr(feature = "serde", serde(default))]
        string description;
        string format;
        #[cfg_attr(feature = "serde", serde(default))]
        Check[][] checks;
        Entry[] params;
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Debug, PartialEq)]
    struct Labels {
        string locale;
        Entry[] items;
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Debug, PartialEq)]
    struct Entry {
        string key;
        string value;
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Debug, PartialEq)]
    struct Check {
        string left;
        #[cfg_attr(feature = "serde", serde(default))]
        string op;
        #[cfg_attr(feature = "serde", serde(default))]
        string right;
    }
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

impl Display {
    pub fn hash_struct(&self) -> B256 {
        self.eip712_hash_struct()
    }
}

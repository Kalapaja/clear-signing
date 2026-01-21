use crate::error::ParseError;
use crate::reference::Reference;
use alloc::string::ToString;
use alloy_core::primitives::B256;
use alloy_core::sol;
use alloy_core::sol_types::SolStruct;
use core::clone::Clone;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplaySpecFile {
    pub displays: alloc::vec::Vec<Display>,
}

sol! {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Display {
        address address;
        string abi;
        #[serde(default)]
        string title;
        #[serde(default)]
        string description;
        Field[] fields;
        #[serde(default)]
        Labels[] labels;
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Field {
        #[serde(default)]
        string title;
        #[serde(default)]
        string description;
        string format;
        #[serde(default)]
        Check[][] checks;
        Entry[] params;
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Labels {
        string locale;
        Entry[] items;
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Entry {
        string key;
        string value;
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Check {
        string left;
        #[serde(default)]
        string op;
        #[serde(default)]
        string right;
    }
}

use crate::fields::Label;

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

    pub fn get_label(&self, label: &str) -> Result<Label, ParseError> {
        let reference = Reference::parse(label)?;

        let label = match &reference {
            Reference::Literal(_) => return Ok(reference),
            Reference::Identifier { identifier, .. } => {
                if &identifier.container != "labels" {
                    return Err(ParseError::SmthWentWrong(
                        "Invalid label container {} ".to_string(),
                    ));
                };

                identifier.only_segments()?
            }
        };

        for labels in &self.labels {
            let locale = &labels.locale;

            let found = labels
                .items
                .iter()
                .map(|entry| &entry.key)
                .any(|key| key == &label);

            if !found {
                return Err(ParseError::LabelNotFound {
                    locale: locale.to_string(),
                    label: reference,
                });
            }
        }

        Ok(reference)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

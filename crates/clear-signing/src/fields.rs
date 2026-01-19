use crate::display::Labels;
use crate::error::ParseError;
use crate::reference::Reference;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloy_core::primitives::{Address, Bytes, I256, U256};
use core::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ClearCall {
    pub title: Label,
    pub description: Label,
    pub payable: bool,
    pub clear: bool,
    pub fields: Vec<DisplayField>,
    pub labels: Vec<Labels>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    In,
    Out,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DisplayField {
    Call {
        title: Label,
        description: Label,
        call: ClearCall,
    },
    Match {
        title: Label,
        description: Label,
        values: Vec<DisplayField>,
    },
    Array {
        title: Label,
        description: Label,
        values: Vec<Vec<DisplayField>>,
    },
    Contract {
        title: Label,
        description: Label,
        contract: Address,
    },
    Token {
        title: Label,
        description: Label,
        token: Address,
    },
    TokenAmount {
        title: Label,
        description: Label,
        token: Address,
        amount: U256,
        direction: Option<Direction>,
    },
    NativeAmount {
        title: Label,
        description: Label,
        amount: U256,
    },
    Boolean {
        title: Label,
        description: Label,
        value: bool,
    },
    Percentage {
        title: Label,
        description: Label,
        value: U256,
        basis: U256,
    },
    Duration {
        title: Label,
        description: Label,
        value: Duration,
    },
    Datetime {
        title: Label,
        description: Label,
        value: Duration,
    },
    Bitmask {
        title: Label,
        description: Label,
        values: Vec<Label>,
    },
    String {
        title: Label,
        description: Label,
        value: String,
    },
    Bytes {
        title: Label,
        description: Label,
        value: Bytes,
    },
    Int {
        title: Label,
        description: Label,
        value: I256,
    },
    Uint {
        title: Label,
        description: Label,
        value: U256,
    },
    Address {
        title: Label,
        description: Label,
        value: Address,
    },
}

pub type Label = Reference;

impl Label {
    pub fn resolve(
        &self,
        labels_dict: &[Labels],
        locale: Option<&str>,
    ) -> Result<String, ParseError> {
        match self {
            Reference::Literal(s) => Ok(s.clone()),
            Reference::Identifier { identifier, .. } => {
                let label = identifier.only_segments()?;

                let target_labels = if let Some(l) = locale {
                    labels_dict.iter().find(|labels| labels.locale == l)
                } else {
                    labels_dict.first()
                };

                if let Some(labels) = target_labels
                    && let Some(entry) = labels.items.iter().find(|entry| entry.key == label)
                {
                    return Ok(entry.value.clone());
                }

                Err(ParseError::LabelNotFound {
                    locale: locale.unwrap_or("default").to_string(),
                    label: self.clone(),
                })
            }
        }
    }
}

use crate::display::Labels;
use crate::error::ParseError;
use alloc::format;
use alloc::string::String;
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

impl Direction {
    pub fn from_str(direction: &str) -> Result<Self, ParseError> {
        match direction {
            "in" => Ok(Direction::In),
            "out" => Ok(Direction::Out),
            _ => Err(ParseError::SmthWentWrong(format!(
                "Unknown direction: {}",
                direction
            ))),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
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
        direction: Option<Direction>,
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

pub type Label = String;

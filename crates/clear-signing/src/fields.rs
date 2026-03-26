use crate::display::Labels;
use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::{Address, Bytes, I256, U256};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct ClearCall {
    pub title: Label,
    pub description: Label,
    pub payable: bool,
    pub clear: bool,
    pub fields: Vec<DisplayField>,
    pub labels: Vec<Labels>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub enum Direction {
    In,
    Out,
}

impl Direction {
    pub fn from_str(direction: &str) -> crate::Result<Self> {
        match direction {
            "in" => Ok(Direction::In),
            "out" => Ok(Direction::Out),
            _ => anyhow::bail!("Unknown direction: {}", direction),
        }
    }

    pub fn try_from_sol_value(value: crate::sol::SolValue) -> crate::Result<Self> {
        Self::from_str(value.as_literal()?.as_str())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TimeUnits {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
}

impl TimeUnits {
    pub fn from_str(s: &str) -> crate::Result<Self> {
        match s {
            "seconds" => Ok(Self::Seconds),
            "minutes" => Ok(Self::Minutes),
            "hours" => Ok(Self::Hours),
            "days" => Ok(Self::Days),
            "weeks" => Ok(Self::Weeks),
            other => anyhow::bail!(
                "Unknown units '{}'. Expected: seconds, minutes, hours, days, weeks",
                other
            ),
        }
    }

    pub fn multiplier(self) -> U256 {
        match self {
            Self::Seconds => U256::from(1u64),
            Self::Minutes => U256::from(60u64),
            Self::Hours => U256::from(3_600u64),
            Self::Days => U256::from(86_400u64),
            Self::Weeks => U256::from(604_800u64),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum DisplayField {
    Call {
        title: Label,
        description: Label,
        call: ClearCall,
    },
    Map {
        title: Label,
        description: Label,
        fields: Vec<DisplayField>,
    },
    Array {
        title: Label,
        description: Label,
        fields: Vec<Vec<DisplayField>>,
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
        token_id: Option<U256>,
    },
    TokenAmount {
        title: Label,
        description: Label,
        token: Address,
        token_id: Option<U256>,
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
        value: U256,
    },
    Datetime {
        title: Label,
        description: Label,
        value: U256,
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
    Units {
        title: Label,
        description: Label,
        value: U256,
        decimals: U256,
    },
    Switch {
        title: Label,
        description: Label,
        fields: Vec<DisplayField>,
    },
}

pub type Label = String;

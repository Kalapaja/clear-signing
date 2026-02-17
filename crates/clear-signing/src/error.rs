use crate::fields::Label;
use alloc::format;
use alloc::string::String;
use alloy_dyn_abi::parser;
use alloy_primitives::hex::FromHexError;
use alloy_primitives::{ruint, Address, Selector, B256};
use nom::{error, Err};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    RecursionLimitExceeded,
    UnknownContract(Address),
    UnknownToken(Address),
    DisplayHashMismatch(B256, B256),
    FunctionNotPayable,
    FunctionNotWriteable,
    DisplayNotFound { address: Address, selector: Selector },
    UnknownFormat(String),
    UnknownOperator(String),
    SmthWentWrong(String),
    ParamNotFound(String),
    LabelNotFound { locale: String, label: Label },
    ReferenceNotFound(String),
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::RecursionLimitExceeded => write!(f, "Max recursion depth exceeded"),
            ParseError::UnknownContract(addr) => write!(f, "Unknown contract: {}", addr),
            ParseError::DisplayHashMismatch(exp, act) => write!(f, "Display hash mismatch, expected: {}, actual: {}", exp, act),
            ParseError::FunctionNotPayable => write!(f, "Function is not payable"),
            ParseError::FunctionNotWriteable => write!(f, "Function is not writeable"),
            ParseError::UnknownFormat(format) => write!(f, "Unknown format: {}", format),
            ParseError::UnknownOperator(op) => write!(f, "Unknown operator: {}", op),
            ParseError::SmthWentWrong(msg) => write!(f, "Smth went wrong: {}", msg),
            ParseError::UnknownToken(token) => write!(f, "Unknown token: {:?}", token),
            ParseError::ReferenceNotFound(msg) => write!(f, "Reference not found: {}", msg),
            ParseError::LabelNotFound { locale, label } => {
                write!(f, "Label no found {} {:?}", locale, label)
            }
            ParseError::DisplayNotFound { address, selector } => {
                write!(f, "Display not found: {:?} at address {}", selector, address)
            }
            ParseError::ParamNotFound(msg) => {
                write!(f, "Param not found: {}", msg)
            }
        }
    }
}

impl From<alloy_sol_types::Error> for ParseError {
    fn from(e: alloy_sol_types::Error) -> Self {
        ParseError::SmthWentWrong(format!("ABI decode error: {}", e))
    }
}

impl From<alloy_dyn_abi::Error> for ParseError {
    fn from(e: alloy_dyn_abi::Error) -> Self {
        ParseError::SmthWentWrong(format!("Dyn ABI error: {}", e))
    }
}

impl From<parser::Error> for ParseError {
    fn from(e: parser::Error) -> Self {
        ParseError::SmthWentWrong(format!("Function parse error: {}", e))
    }
}

impl From<alloy_primitives::ParseSignedError> for ParseError {
    fn from(e: alloy_primitives::ParseSignedError) -> Self {
        ParseError::SmthWentWrong(format!("ParseSignedError: {}", e))
    }
}

impl From<FromHexError> for ParseError {
    fn from(e: FromHexError) -> Self {
        ParseError::SmthWentWrong(format!("Invalid hex: {}", e))
    }
}

impl From<ruint::ParseError> for ParseError {
    fn from(e: ruint::ParseError) -> Self {
        ParseError::SmthWentWrong(format!("ParseError: {}", e))
    }
}

impl From<ruint::FromUintError<u64>> for ParseError {
    fn from(e: ruint::FromUintError<u64>) -> Self {
        ParseError::SmthWentWrong(format!("Can't parse uint into u64: {}", e))
    }
}

impl From<core::num::ParseIntError> for ParseError {
    fn from(e: core::num::ParseIntError) -> Self {
        ParseError::SmthWentWrong(format!("Can't parse int: {}", e))
    }
}

impl From<Err<error::Error<&str>>> for ParseError {
    fn from(e: Err<error::Error<&str>>) -> Self {
        ParseError::SmthWentWrong(format!("Nom error: {}", e))
    }
}

#[cfg(feature = "serde_json")]
impl From<serde_json::Error> for ParseError {
    fn from(e: serde_json::Error) -> Self {
        ParseError::SmthWentWrong(format!("Json error: {}", e))
    }
}

impl From<core::array::TryFromSliceError> for ParseError {
    fn from(e: core::array::TryFromSliceError) -> Self {
        ParseError::SmthWentWrong(format!("TryFromSliceError: {}", e))
    }
}

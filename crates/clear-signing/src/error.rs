use crate::fields::Label;
use alloc::format;
use alloc::string::String;
use alloy_core::dyn_abi::parser;
use alloy_core::hex::FromHexError;
use alloy_core::primitives::{ruint, Address, Selector};
use nom::{error, Err};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    RecursionLimitExceeded,
    UnknownContract(Address),
    UnknownToken(Address),
    DisplayHashMismatch,
    FunctionNotPayable,
    FunctionNotWriteable,
    DisplayNotFound { selector: Selector },
    UnknownFormat(String),
    SmthWentWrong(String),
    ParamNotFound(String),
    LabelNotFound { locale: String, label: Label },
    ReferenceNotFound(String),
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::RecursionLimitExceeded => {
                write!(f, "Max recursion depth exceeded")
            }
            ParseError::UnknownContract(addr) => {
                write!(f, "Unknown contract or token: {}", addr)
            }
            ParseError::DisplayHashMismatch => {
                write!(f, "Display hash mismatch",)
            }
            ParseError::FunctionNotPayable => {
                write!(f, "Function is not payable")
            }
            ParseError::FunctionNotWriteable => {
                write!(f, "Function is not writeable")
            }
            ParseError::UnknownFormat(format) => write!(f, "Unknown format: {}", format),
            ParseError::SmthWentWrong(msg) => write!(f, "Smth went wrong: {}", msg),
            ParseError::UnknownToken(token) => write!(f, "Unknown token: {:?}", token),
            ParseError::LabelNotFound { locale, label } => {
                write!(f, "Label no found {} {:?}", locale, label)
            }
            ParseError::ReferenceNotFound(msg) => write!(f, "Reference not found: {}", msg),
            ParseError::DisplayNotFound { selector } => {
                write!(f, "Display not found: {:?}", selector)
            },
            ParseError::ParamNotFound(msg) => {
                write!(f, "Param not found: {}", msg)
            }
        }
    }
}

impl From<alloy_core::sol_types::Error> for ParseError {
    fn from(e: alloy_core::sol_types::Error) -> Self {
        ParseError::SmthWentWrong(format!("ABI decode error: {}", e))
    }
}

impl From<alloy_core::dyn_abi::Error> for ParseError {
    fn from(e: alloy_core::dyn_abi::Error) -> Self {
        ParseError::SmthWentWrong(format!("Dyn ABI error: {}", e))
    }
}

impl From<alloy_core::json_abi::Error> for ParseError {
    fn from(e: alloy_core::json_abi::Error) -> Self {
        ParseError::SmthWentWrong(format!("JSON ABI error: {:?}", e))
    }
}

impl From<parser::Error> for ParseError {
    fn from(e: parser::Error) -> Self {
        ParseError::SmthWentWrong(format!("Function parse error: {}", e))
    }
}

impl From<alloy_core::primitives::ParseSignedError> for ParseError {
    fn from(e: alloy_core::primitives::ParseSignedError) -> Self {
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

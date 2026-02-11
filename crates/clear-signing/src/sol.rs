use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};
use alloy_primitives::{keccak256, Address, Function, Selector, I256, U256};
use alloy_dyn_abi::{DynSolType, DynSolValue, Word};
use nom::{
    branch::alt, bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{all_consuming, map, map_res, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded},
    IResult,
    Parser,
};

use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolType {
    /// Boolean.
    Bool,
    /// Signed Integer.
    Int(usize),
    /// Unsigned Integer.
    Uint(usize),
    /// Fixed-size bytes, up to 32.
    FixedBytes(usize),
    /// Address.
    Address,
    /// Function.
    Function,

    /// Dynamic bytes.
    Bytes,
    /// String.
    String,

    /// Dynamically sized array.
    Array(Box<Self>),
    /// Fixed-sized array.
    FixedArray(Box<Self>, usize),
    /// Tuple.
    Tuple(Vec<(Option<String>, Self)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMutability {
    Pure,
    View,
    Payable,
    NonPayable,
}

impl SolType {
    pub fn sol_type_name(&self) -> String {
        match self {
            SolType::Bool => "bool".to_string(),
            SolType::Int(size) => format!("int{}", size),
            SolType::Uint(size) => format!("uint{}", size),
            SolType::FixedBytes(size) => format!("bytes{}", size),
            SolType::Address => "address".to_string(),
            SolType::Function => "function".to_string(),
            SolType::Bytes => "bytes".to_string(),
            SolType::String => "string".to_string(),
            SolType::Array(inner) => format!("{}[]", inner.sol_type_name()),
            SolType::FixedArray(inner, size) => format!("{}[{}]", inner.sol_type_name(), size),
            SolType::Tuple(inner) => {
                let types: Vec<String> = inner.iter().map(|(_, t)| t.sol_type_name()).collect();
                format!("({})", types.join(","))
            }
        }
    }

    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let (_, sol_type) = all_consuming(parse_tuple).parse(input)?;
        Ok(sol_type)
    }
}

impl From<&SolType> for DynSolType {
    fn from(value: &SolType) -> Self {
        match value {
            SolType::Bool => DynSolType::Bool,
            SolType::Int(size) => DynSolType::Int(*size),
            SolType::Uint(size) => DynSolType::Uint(*size),
            SolType::FixedBytes(size) => DynSolType::FixedBytes(*size),
            SolType::Address => DynSolType::Address,
            SolType::Function => DynSolType::Function,
            SolType::Bytes => DynSolType::Bytes,
            SolType::String => DynSolType::String,
            SolType::Array(inner) => DynSolType::Array(Box::new(Self::from(&**inner))),
            SolType::FixedArray(inner, size) => {
                DynSolType::FixedArray(Box::new(Self::from(&**inner)), *size)
            }
            SolType::Tuple(inner) => {
                DynSolType::Tuple(inner.iter().map(|(_, t)| Self::from(t)).collect())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SolValue {
    Literal(String),
    /// A boolean.
    Bool(bool),
    /// A signed integer. The second parameter is the number of bits, not bytes.
    Int(I256, usize),
    /// An unsigned integer. The second parameter is the number of bits, not bytes.
    Uint(U256, usize),
    /// A fixed-length byte array. The second parameter is the number of bytes.
    FixedBytes(Word, usize),
    /// An address.
    Address(Address),
    /// A function pointer.
    Function(Function),

    /// A dynamic-length byte array.
    Bytes(Vec<u8>),
    /// A string.
    String(String),

    /// A dynamically-sized array of values.
    Array(Vec<Self>),
    /// A fixed-size array of values.
    FixedArray(Vec<Self>),
    /// A tuple of values.
    Tuple(Vec<(Option<String>, Self)>),
}

impl SolValue {
    pub fn matches(&self, other: &SolValue) -> Result<bool, ParseError> {
        match (self, other) {
            (SolValue::Literal(l1), SolValue::Literal(l2)) => Ok(l1 == l2),
            (SolValue::Bool(v1), SolValue::Bool(v2)) => Ok(v1 == v2),
            (SolValue::Int(v1, _), SolValue::Int(v2, _)) => Ok(v1 == v2),
            (SolValue::Uint(v1, _), SolValue::Uint(v2, _)) => Ok(v1 == v2),
            (SolValue::FixedBytes(v1, _), SolValue::FixedBytes(v2, _)) => Ok(v1 == v2),
            (SolValue::Address(v1), SolValue::Address(v2)) => Ok(v1 == v2),
            (SolValue::Bytes(v1), SolValue::Bytes(v2)) => Ok(v1 == v2),
            (SolValue::String(v1), SolValue::String(v2)) => Ok(v1 == v2),
            (SolValue::Tuple(v1), SolValue::Tuple(v2)) => {
                if v1.len() != v2.len() {
                    Ok(false)
                } else {
                    for ((_, val1), (_, val2)) in v1.iter().zip(v2.iter()) {
                        if !val1.matches(val2)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
            }
            (SolValue::Array(v1), SolValue::Array(v2)) => {
                if v1.len() != v2.len() {
                    Ok(false)
                } else {
                    for (val1, val2) in v1.iter().zip(v2.iter()) {
                        if !val1.matches(val2)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
            }

            // Literal matching logic
            (SolValue::Bool(v), SolValue::Literal(_)) => Ok(v == &other.as_bool()?),
            (SolValue::Int(v, _), SolValue::Literal(_)) => Ok(v == &other.as_int()?),
            (SolValue::Uint(v, _), SolValue::Literal(_)) => Ok(v == &other.as_uint()?),
            (SolValue::FixedBytes(_v, _), SolValue::Literal(_)) => {
                Ok(self.as_bytes()? == other.as_bytes()?)
            }
            (SolValue::Address(v), SolValue::Literal(_)) => Ok(v == &other.as_address()?),
            (SolValue::Bytes(v), SolValue::Literal(_)) => Ok(v == &other.as_bytes()?),
            (SolValue::String(v), SolValue::Literal(_)) => Ok(v == &other.as_string()?),

            (SolValue::Literal(_), SolValue::Bool(v)) => Ok(&self.as_bool()? == v),
            (SolValue::Literal(_), SolValue::Int(v, _)) => Ok(&self.as_int()? == v),
            (SolValue::Literal(_), SolValue::Uint(v, _)) => Ok(&self.as_uint()? == v),
            (SolValue::Literal(_), SolValue::FixedBytes(_v, _)) => {
                Ok(self.as_bytes()? == other.as_bytes()?)
            }
            (SolValue::Literal(_), SolValue::Address(v)) => Ok(&self.as_address()? == v),
            (SolValue::Literal(_), SolValue::Bytes(v)) => Ok(&self.as_bytes()? == v),
            (SolValue::Literal(_), SolValue::String(v)) => Ok(&self.as_string()? == v),

            _ => Ok(false),
        }
    }

    pub fn as_address(&self) -> Result<Address, ParseError> {
        match self {
            SolValue::Address(a) => Ok(*a),
            SolValue::Bytes(bytes) => {
                if bytes.len() != 20 {
                    return Err(ParseError::SmthWentWrong(format!(
                        "Invalid address length: {}",
                        bytes.len()
                    )));
                }

                Ok(Address::from_slice(bytes))
            }
            SolValue::Literal(string) => Ok(string.parse()?),
            _ => Err(ParseError::SmthWentWrong(format!(
                "Type mismatch: expected address, got {:?}",
                self
            ))),
        }
    }

    pub fn as_uint(&self) -> Result<U256, ParseError> {
        match self {
            SolValue::Uint(v, _) => Ok(*v),
            SolValue::Bytes(bytes) => {
                if bytes.len() > 32 {
                    return Err(ParseError::SmthWentWrong(format!(
                        "Bytes length {} exceeds 32 for uint conversion",
                        bytes.len()
                    )));
                }
                Ok(U256::from_be_slice(bytes))
            }
            SolValue::Literal(string) => Ok(string.parse()?),
            _ => Err(ParseError::SmthWentWrong(format!(
                "Type mismatch: expected uint, got {:?}",
                self
            ))),
        }
    }

    pub fn as_literal(&self) -> Result<String, ParseError> {
        match self {
            SolValue::Literal(string) => Ok(string.clone()),
            _ => Err(ParseError::SmthWentWrong(format!(
                "Type mismatch: expected literal, got {:?}",
                self
            ))),
        }
    }

    pub fn as_int(&self) -> Result<I256, ParseError> {
        match self {
            SolValue::Int(v, _) => Ok(*v),
            SolValue::Literal(string) => Ok(string.parse()?),
            _ => Err(ParseError::SmthWentWrong(format!(
                "Type mismatch: expected int, got {:?}",
                self
            ))),
        }
    }

    pub fn as_bool(&self) -> Result<bool, ParseError> {
        match self {
            SolValue::Bool(v) => Ok(*v),
            SolValue::Literal(string) => match string.to_lowercase().as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(ParseError::SmthWentWrong(format!(
                    "Invalid boolean literal: {}",
                    string
                ))),
            },
            _ => Err(ParseError::SmthWentWrong(format!(
                "Type mismatch: expected bool, got {:?}",
                self
            ))),
        }
    }

    pub fn as_string(&self) -> Result<String, ParseError> {
        match self {
            SolValue::String(s) => Ok(s.clone()),
            SolValue::Literal(string) => Ok(string.clone()),
            _ => Err(ParseError::SmthWentWrong(format!(
                "Type mismatch: expected string, got {:?}",
                self
            ))),
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, ParseError> {
        match self {
            SolValue::Bytes(b) => Ok(b.clone()),
            SolValue::FixedBytes(b, size) => Ok(b[..*size].to_vec()),
            SolValue::Uint(val, size) => {
                let byte_len = size / 8;
                let bytes = val.to_be_bytes::<32>();
                Ok(bytes[32 - byte_len..].to_vec())
            }
            SolValue::Literal(string) => {
                Ok(alloy_primitives::hex::decode(
                    string.trim_start_matches("0x"),
                )?)
            }
            _ => Err(ParseError::SmthWentWrong(format!(
                "Type mismatch: expected bytes, got {:?}",
                self
            ))),
        }
    }

    pub fn as_array(&self) -> Result<Vec<SolValue>, ParseError> {
        match self {
            SolValue::Array(arr) => Ok(arr.clone()),
            SolValue::FixedArray(arr) => Ok(arr.clone()),
            SolValue::Bytes(bytes) => Ok(bytes
                .iter()
                .map(|b| {
                    let mut word = Word::ZERO;
                    word[0] = *b;
                    SolValue::FixedBytes(word, 1)
                })
                .collect()),
            SolValue::FixedBytes(word, size) => Ok(word[..*size]
                .iter()
                .map(|b| {
                    let mut word = Word::ZERO;
                    word[0] = *b;
                    SolValue::FixedBytes(word, 1)
                })
                .collect()),
            SolValue::Uint(val, size) => {
                let byte_len = size / 8;
                let bytes = val.to_be_bytes::<32>();
                Ok(bytes[32 - byte_len..]
                    .iter()
                    .map(|b| {
                        let mut word = Word::ZERO;
                        word[0] = *b;
                        SolValue::FixedBytes(word, 1)
                    })
                    .collect())
            }
            _ => Err(ParseError::SmthWentWrong(format!(
                "Type mismatch: expected array, got {:?}",
                self
            ))),
        }
    }

    pub fn from(value: DynSolValue, ty: &SolType) -> Result<SolValue, ParseError> {
        match value {
            DynSolValue::Bool(v) => Ok(SolValue::Bool(v)),
            DynSolValue::Int(v, size) => Ok(SolValue::Int(v, size)),
            DynSolValue::Uint(v, size) => Ok(SolValue::Uint(v, size)),
            DynSolValue::FixedBytes(v, size) => Ok(SolValue::FixedBytes(v, size)),
            DynSolValue::Address(v) => Ok(SolValue::Address(v)),
            DynSolValue::Function(v) => Ok(SolValue::Function(v)),
            DynSolValue::Bytes(v) => Ok(SolValue::Bytes(v)),
            DynSolValue::String(v) => Ok(SolValue::String(v)),
            DynSolValue::Array(values) => {
                if let SolType::Array(inner_type) = ty {
                    let converted: Result<Vec<_>, _> = values
                        .into_iter()
                        .map(|v| Self::from(v, inner_type))
                        .collect();
                    Ok(SolValue::Array(converted?))
                } else {
                    Err(ParseError::SmthWentWrong(format!(
                        "Type mismatch: expected Array, got {:?}",
                        ty
                    )))
                }
            }
            DynSolValue::FixedArray(values) => {
                if let SolType::FixedArray(inner_type, _) = ty {
                    let converted: Result<Vec<_>, _> = values
                        .into_iter()
                        .map(|v| Self::from(v, inner_type))
                        .collect();
                    Ok(SolValue::FixedArray(converted?))
                } else {
                    Err(ParseError::SmthWentWrong(format!(
                        "Type mismatch: expected FixedArray, got {:?}",
                        ty
                    )))
                }
            }
            DynSolValue::Tuple(values) => {
                if let SolType::Tuple(types) = ty {
                    if values.len() != types.len() {
                        return Err(ParseError::SmthWentWrong(format!(
                            "Tuple length mismatch: values {} != types {}",
                            values.len(),
                            types.len()
                        )));
                    }

                    let entries = values
                        .into_iter()
                        .zip(types)
                        .map(|(val, (name_opt, type_def))| {
                            Ok((name_opt.clone(), Self::from(val, type_def)?))
                        })
                        .collect::<Result<Vec<_>, ParseError>>()?;

                    Ok(SolValue::Tuple(entries))
                } else {
                    Err(ParseError::SmthWentWrong(format!(
                        "Type mismatch: expected Tuple, got {:?}",
                        ty
                    )))
                }
            }
        }
    }
}

pub struct SolFunction {
    pub name: String,
    pub tuple: SolType,
    pub state_mutability: StateMutability,
}

impl SolFunction {
    pub fn parse(signature: &str) -> Result<Self, ParseError> {
        let input = signature.trim();

        let (input, _) = ws::<_, nom::error::Error<&str>, _>(tag("function")).parse(input)?;

        let (input, function_name) = ws(identifier).parse(input)?;

        let (input, tuple) = parse_tuple(input)?;

        let (input, state_mutability) = parse_state_mutability(input)?;

        if !input.trim().is_empty() {
            return Err(ParseError::SmthWentWrong(format!(
                "Trailing data: {}",
                input
            )));
        }

        Ok(SolFunction {
            name: function_name.to_string(),
            tuple,
            state_mutability,
        })
    }

    pub fn selector(&self) -> Selector {
        let type_name = self.tuple.sol_type_name();
        let signature = format!("{}{}", self.name, type_name);
        let hash = keccak256(signature.as_bytes());
        Selector::from_slice(&hash[..4])
    }

    pub fn decode(&self, data: &[u8]) -> Result<SolValue, ParseError> {
        if data.len() < 4 || &data[..4] != self.selector().as_slice() {
            return Err(ParseError::SmthWentWrong(
                "Invalid data, wrong selector".to_string(),
            ));
        }
        let ty = DynSolType::from(&self.tuple);
        let decoded = ty.abi_decode_params(&data[4..])?;
        SolValue::from(decoded, &self.tuple)
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

fn is_ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(take_while1(is_ident_start), take_while(is_ident_part))).parse(input)
}

fn ws<'a, O, E, P>(parser: P) -> impl Parser<&'a str, Output = O, Error = E>
where
    P: Parser<&'a str, Output = O, Error = E>,
    E: nom::error::ParseError<&'a str>,
{
    delimited(multispace0, parser, multispace0)
}

fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>()).parse(input)
}

fn parse_param(input: &str) -> IResult<&str, (Option<String>, SolType)> {
    let (input, sol_type) = parse_type_def.parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, name) = opt(identifier).parse(input)?;

    Ok((input, (name.map(|s| s.to_string()), sol_type)))
}

fn parse_state_mutability(input: &str) -> IResult<&str, StateMutability> {
    alt((
        value(StateMutability::Pure, ws(tag("pure"))),
        value(StateMutability::View, ws(tag("view"))),
        value(StateMutability::Payable, ws(tag("payable"))),
        value(StateMutability::NonPayable, ws(tag("nonpayable"))),
        |i| Ok((i, StateMutability::NonPayable)),
    ))
    .parse(input)
}

fn parse_tuple(input: &str) -> IResult<&str, SolType> {
    map(
        delimited(
            char('('),
            ws(separated_list0(char(','), ws(parse_param))),
            char(')'),
        ),
        SolType::Tuple,
    )
    .parse(input)
}

fn parse_base_type(input: &str) -> IResult<&str, SolType> {
    alt((
        value(SolType::Bool, tag("bool")),
        value(SolType::Address, tag("address")),
        value(SolType::String, tag("string")),
        value(SolType::Function, tag("function")),
        map(preceded(tag("bytes"), parse_usize), SolType::FixedBytes),
        value(SolType::Bytes, tag("bytes")),
        map(preceded(tag("uint"), opt(parse_usize)), |sz| {
            SolType::Uint(sz.unwrap_or(256))
        }),
        map(preceded(tag("int"), opt(parse_usize)), |sz| {
            SolType::Int(sz.unwrap_or(256))
        }),
        parse_tuple,
    ))
    .parse(input)
}

fn parse_type_def(input: &str) -> IResult<&str, SolType> {
    let (input, mut sol_type) = parse_base_type(input)?;

    let (input, suffixes) = many0(ws(alt((
        map(tag("[]"), |_| None),
        map(delimited(char('['), parse_usize, char(']')), Some),
    ))))
    .parse(input)?;

    for size in suffixes {
        match size {
            Some(n) => sol_type = SolType::FixedArray(Box::new(sol_type), n),
            None => sol_type = SolType::Array(Box::new(sol_type)),
        }
    }

    Ok((input, sol_type))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_simple_sol_type() {
        let sig = "function foo(uint256 a, string b)";
        let func = SolFunction::parse(sig).unwrap();
        if let SolType::Tuple(params) = func.tuple {
            assert_eq!(params[0].0.as_deref(), Some("a"));
            assert!(matches!(params[0].1, SolType::Uint(256)));
            assert_eq!(params[1].0.as_deref(), Some("b"));
            assert!(matches!(params[1].1, SolType::String));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_user_signature_sol_type() {
        let sig = "function doSomething(address add, uint256, (bytes32[] hashes, bool, bytes[] datas)[][] calls)";
        let func = SolFunction::parse(sig).expect("Failed to parse");

        assert_eq!(func.name, "doSomething");

        match func.tuple {
            SolType::Tuple(params) => {
                assert_eq!(params.len(), 3);

                // 1. address add
                assert_eq!(params[0].0.as_deref(), Some("add"));
                assert!(matches!(params[0].1, SolType::Address));

                // 2. uint256
                assert_eq!(params[1].0, None);
                assert!(matches!(params[1].1, SolType::Uint(256)));

                // 3. tuple[][] calls
                assert_eq!(params[2].0.as_deref(), Some("calls"));

                // Verify array structure: Array(Array(Tuple(...)))
                if let SolType::Array(inner) = &params[2].1 {
                    if let SolType::Array(inner2) = &**inner {
                        if let SolType::Tuple(tuple_params) = &**inner2 {
                            assert_eq!(tuple_params.len(), 3);
                            assert_eq!(tuple_params[0].0.as_deref(), Some("hashes"));
                            // bytes32[] hashes
                            match &tuple_params[0].1 {
                                SolType::Array(b) => {
                                    assert!(matches!(**b, SolType::FixedBytes(32)))
                                }
                                _ => panic!("Expected Array(FixedBytes(32)) for hashes"),
                            }

                            assert_eq!(tuple_params[1].0, None);
                            assert!(matches!(tuple_params[1].1, SolType::Bool));

                            assert_eq!(tuple_params[2].0.as_deref(), Some("datas"));
                            match &tuple_params[2].1 {
                                SolType::Array(b) => assert!(matches!(**b, SolType::Bytes)),
                                _ => panic!("Expected Array(Bytes) for datas"),
                            }
                        } else {
                            panic!("Inner type should be tuple");
                        }
                    } else {
                        panic!("Expected 2D array");
                    }
                } else {
                    panic!("Expected array of array");
                }
            }
            _ => panic!("Expected top level Tuple"),
        }
    }

    #[test]
    fn test_complex_selector() {
        // function doSomething(address add, uint256, (bytes32[] hashes, bool, bytes[] datas)[][] calls)
        let sig = "function doSomething(address add, uint256, (bytes32[] hashes, bool, bytes[] datas)[][] calls)";
        let func = SolFunction::parse(sig).expect("Failed to parse");

        let canonical = "doSomething(address,uint256,(bytes32[],bool,bytes[])[][])";
        let hash = keccak256(canonical);
        assert_eq!(func.selector(), Selector::from_slice(&hash[..4]));
    }

    #[test]
    fn test_selector() {
        let sig = "function foo(uint256 a, string b)";
        let func = SolFunction::parse(sig).unwrap();
        // foo(uint256,string) -> 0x06023348
        // Keccak256("foo(uint256,string)") = 06023348...
        let hash = keccak256("foo(uint256,string)");
        assert_eq!(func.selector(), Selector::from_slice(&hash[..4]));
    }

    #[test]
    fn test_parse_pure_function() {
        let sig = "function foo(uint256) pure";
        let func = SolFunction::parse(sig).unwrap();
        assert_eq!(func.state_mutability, StateMutability::Pure);
    }

    #[test]
    fn test_parse_view_function() {
        let sig = "function foo(uint256) view";
        let func = SolFunction::parse(sig).unwrap();
        assert_eq!(func.state_mutability, StateMutability::View);
    }

    #[test]
    fn test_parse_payable_function() {
        let sig = "function foo(uint256) payable";
        let func = SolFunction::parse(sig).unwrap();
        assert_eq!(func.state_mutability, StateMutability::Payable);
    }

    #[test]
    fn test_parse_default_nonpayable_function() {
        let sig = "function foo(uint256)";
        let func = SolFunction::parse(sig).unwrap();
        assert_eq!(func.state_mutability, StateMutability::NonPayable);
    }

    #[test]
    fn test_sol_type_parse_tuple() {
        let input = "(address recipient, uint256 amount)";
        let sol_type = SolType::parse(input).unwrap();
        if let SolType::Tuple(params) = sol_type {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].0.as_deref(), Some("recipient"));
            assert!(matches!(params[0].1, SolType::Address));
            assert_eq!(params[1].0.as_deref(), Some("amount"));
            assert!(matches!(params[1].1, SolType::Uint(256)));
        } else {
            panic!("Expected Tuple");
        }

        // Must fail without root ()
        assert!(SolType::parse("uint256 amount").is_err());
    }

    #[test]
    fn test_sol_type_parse_nested_tuple() {
        let input = "(address to, (uint256 amount, string memo) detail)";
        let sol_type = SolType::parse(input).unwrap();
        if let SolType::Tuple(params) = sol_type {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].0.as_deref(), Some("to"));
            if let SolType::Tuple(inner_params) = &params[1].1 {
                assert_eq!(params[1].0.as_deref(), Some("detail"));
                assert_eq!(inner_params.len(), 2);
                assert_eq!(inner_params[0].0.as_deref(), Some("amount"));
                assert_eq!(inner_params[1].0.as_deref(), Some("memo"));
            } else {
                panic!("Expected nested Tuple");
            }
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_as_array_array() {
        let val = SolValue::Array(vec![SolValue::Bool(true), SolValue::Bool(false)]);
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], SolValue::Bool(true));
        assert_eq!(arr[1], SolValue::Bool(false));
    }

    #[test]
    fn test_as_array_fixed_array() {
        let val = SolValue::FixedArray(vec![SolValue::Bool(true), SolValue::Bool(false)]);
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], SolValue::Bool(true));
        assert_eq!(arr[1], SolValue::Bool(false));
    }

    #[test]
    fn test_as_array_bytes() {
        let val = SolValue::Bytes(vec![0x01, 0x02]);
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        // Check first byte
        if let SolValue::FixedBytes(word, size) = &arr[0] {
            assert_eq!(*size, 1);
            assert_eq!(word[0], 0x01);
        } else {
            panic!("Expected FixedBytes");
        }
    }

    #[test]
    fn test_as_array_fixed_bytes() {
        let mut word = Word::ZERO;
        word[0] = 0x01;
        word[1] = 0x02;
        let val = SolValue::FixedBytes(word, 2);
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        if let SolValue::FixedBytes(w, size) = &arr[0] {
            assert_eq!(*size, 1);
            assert_eq!(w[0], 0x01);
        } else {
            panic!("Expected FixedBytes");
        }
    }

    #[test]
    fn test_as_array_uint() {
        let val = SolValue::Uint(U256::from(0x0102), 256);
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 32);

        // 0x0102 is ...000102. So last two bytes are 0x01 and 0x02.
        // It's big endian. So arr[30] == 0x01, arr[31] == 0x02.
        if let SolValue::FixedBytes(word, size) = &arr[30] {
            assert_eq!(*size, 1);
            assert_eq!(word[0], 0x01);
        } else {
            panic!("Expected FixedBytes");
        }
        if let SolValue::FixedBytes(word, size) = &arr[31] {
            assert_eq!(*size, 1);
            assert_eq!(word[0], 0x02);
        } else {
            panic!("Expected FixedBytes");
        }

        // Test with smaller size (uint16)
        let val_16 = SolValue::Uint(U256::from(0x0304), 16);
        let arr_16 = val_16.as_array().unwrap();
        assert_eq!(arr_16.len(), 2);
        if let SolValue::FixedBytes(word, size) = &arr_16[0] {
            assert_eq!(*size, 1);
            assert_eq!(word[0], 0x03);
        } else {
            panic!("Expected FixedBytes");
        }
        if let SolValue::FixedBytes(word, size) = &arr_16[1] {
            assert_eq!(*size, 1);
            assert_eq!(word[0], 0x04);
        } else {
            panic!("Expected FixedBytes");
        }
    }
    #[test]
    fn test_as_address() {
        let addr = Address::from_slice(&[1u8; 20]);
        let val = SolValue::Address(addr);
        assert_eq!(val.as_address().unwrap(), addr);

        let val_lit = SolValue::Literal(format!("{:?}", addr));
        assert_eq!(val_lit.as_address().unwrap(), addr);

        let val_err = SolValue::Bool(true);
        assert!(val_err.as_address().is_err());
    }

    #[test]
    fn test_as_uint() {
        let val = SolValue::Uint(U256::from(123), 256);
        assert_eq!(val.as_uint().unwrap(), U256::from(123));

        let val_lit = SolValue::Literal("123".to_string());
        assert_eq!(val_lit.as_uint().unwrap(), U256::from(123));

        let val_err = SolValue::Bool(true);
        assert!(val_err.as_uint().is_err());
    }

    #[test]
    fn test_as_int() {
        let val = SolValue::Int(I256::try_from(123).unwrap(), 256);
        assert_eq!(val.as_int().unwrap(), I256::try_from(123).unwrap());

        let val_lit = SolValue::Literal("123".to_string());
        assert_eq!(val_lit.as_int().unwrap(), I256::try_from(123).unwrap());

        let val_err = SolValue::Bool(true);
        assert!(val_err.as_int().is_err());
    }

    #[test]
    fn test_as_bool() {
        let val = SolValue::Bool(true);
        assert!(val.as_bool().unwrap());

        let val_lit_true = SolValue::Literal("true".to_string());
        assert!(val_lit_true.as_bool().unwrap());

        let val_lit_false = SolValue::Literal("false".to_string());
        assert!(!val_lit_false.as_bool().unwrap());

        let val_err = SolValue::Uint(U256::from(1), 256);
        assert!(val_err.as_bool().is_err());
    }

    #[test]
    fn test_as_string() {
        let val = SolValue::String("hello".to_string());
        assert_eq!(val.as_string().unwrap(), "hello");

        let val_lit = SolValue::Literal("world".to_string());
        assert_eq!(val_lit.as_string().unwrap(), "world");

        let val_err = SolValue::Bool(true);
        assert!(val_err.as_string().is_err());
    }

    #[test]
    fn test_as_bytes() {
        let bytes = vec![0x01, 0x02, 0x03];
        let val = SolValue::Bytes(bytes.clone());
        assert_eq!(val.as_bytes().unwrap(), bytes);

        let mut word = Word::ZERO;
        word[0] = 0x01;
        word[1] = 0x02;
        let val_fixed = SolValue::FixedBytes(word, 2);
        assert_eq!(val_fixed.as_bytes().unwrap(), vec![0x01, 0x02]);

        // Test Uint with as_bytes
        let val_uint = SolValue::Uint(U256::from(0x010203), 24);
        assert_eq!(val_uint.as_bytes().unwrap(), vec![0x01, 0x02, 0x03]);

        let val_uint_16 = SolValue::Uint(U256::from(0x0102), 16);
        assert_eq!(val_uint_16.as_bytes().unwrap(), vec![0x01, 0x02]);

        let val_lit = SolValue::Literal("0x010203".to_string());
        assert_eq!(val_lit.as_bytes().unwrap(), bytes);

        let val_err = SolValue::Bool(true);
        assert!(val_err.as_bytes().is_err());
    }

    #[test]
    fn test_tuple_matches() {
        let t1 = SolValue::Tuple(vec![(None, SolValue::Bool(true))]);
        let t2 = SolValue::Tuple(vec![(None, SolValue::Bool(true))]);
        assert!(t1.matches(&t2).unwrap());

        let t3 = SolValue::Tuple(vec![(None, SolValue::Literal("true".to_string()))]);
        assert!(t3.matches(&t1).unwrap());
        assert!(t1.matches(&t3).unwrap());

        let t4 = SolValue::Tuple(vec![(None, SolValue::Bool(false))]);
        assert!(!t1.matches(&t4).unwrap());
    }

    #[test]
    fn test_array_matches() {
        let a1 = SolValue::Array(vec![SolValue::Uint(U256::from(1), 256)]);
        let a2 = SolValue::Array(vec![SolValue::Uint(U256::from(1), 256)]);
        assert!(a1.matches(&a2).unwrap());

        let a3 = SolValue::Array(vec![SolValue::Literal("1".to_string())]);
        assert!(a3.matches(&a1).unwrap());
        assert!(a1.matches(&a3).unwrap());

        let a4 = SolValue::Array(vec![SolValue::Uint(U256::from(2), 256)]);
        assert!(!a1.matches(&a4).unwrap());
    }
}

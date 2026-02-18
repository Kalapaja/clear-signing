use crate::error::ParseError;
use crate::reference::{Index, Member, Reference, Segment, Slice};
use crate::sol::SolValue;
use alloc::string::ToString;
use alloy_primitives::{Address, Bytes, FixedBytes, U256};

const CONTAINER_MSG: &str = "msg";
const CONTAINER_DATA: &str = "data";

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: Address,
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
}

impl Message {
    pub fn new(sender: Address, to: Address, value: U256, data: Bytes) -> Self {
        Self {
            sender,
            to,
            value,
            data,
        }
    }

    pub fn replace_data(&self, data: Bytes) -> Self {
        Self {
            sender: self.sender,
            to: self.to,
            value: self.value,
            data,
        }
    }

    pub fn selector(&self) -> Result<FixedBytes<4>, ParseError> {
        if self.data.len() >= 4 {
            Ok(self.data[..4].try_into().map(FixedBytes::new)?)
        } else {
            Err(ParseError::SmthWentWrong(
                "Invalid message selector".to_string(),
            ))
        }
    }
}

pub fn resolve_value(
    reference: &str,
    message: &Message,
    data: &SolValue,
) -> Result<SolValue, ParseError> {
    match Reference::parse(reference)? {
        Reference::Literal(val) => Ok(SolValue::Literal(val)),
        Reference::Identifier {
            identifier,
            reference: _,
        } => match identifier.container.as_str() {
            CONTAINER_MSG => {
                let value = resolve_msg(&identifier.members, message)?;
                Ok(value)
            }
            CONTAINER_DATA => {
                let value = resolve_data(&identifier.members, data)?;
                Ok(value)
            }
            _ => Err(ParseError::SmthWentWrong(alloc::format!(
                "Invalid variable reference container: {}. Valid containers: ${}, ${}",
                identifier.container,
                CONTAINER_MSG,
                CONTAINER_DATA
            ))),
        },
    }
}

fn resolve_msg(members: &[Member], message: &Message) -> Result<SolValue, ParseError> {
    if !members.len() == 1 {
        return Err(ParseError::SmthWentWrong(alloc::format!(
            "Message path must have exactly one field, got {}",
            members.len()
        )));
    }

    if let Some(Member::Segment(Segment(name))) = members.first() {
        match name.as_str() {
            "sender" => Ok(SolValue::Address(message.sender)),
            "to" => Ok(SolValue::Address(message.to)),
            "value" => Ok(SolValue::Uint(message.value, 256)),
            "data" => Ok(SolValue::Bytes(message.data.to_vec())),
            _ => Err(ParseError::SmthWentWrong(alloc::format!(
                "Unknown message field '$msg.{}'. Available: $msg.sender, $msg.to, $msg.value, $msg.data",
                name
            ))),
        }
    } else {
        Err(ParseError::SmthWentWrong(
            "Message path must have a field name".into(),
        ))
    }
}

fn resolve_data(members: &[Member], data: &SolValue) -> Result<SolValue, ParseError> {
    let mut path = members.iter();

    let mut value = if let Some(Member::Segment(segment)) = path.next() {
        parse_segment(data, segment)?
    } else {
        return Err(ParseError::SmthWentWrong(
            "Parameter path must have a field name".into(),
        ));
    };

    for seg in path {
        value = match seg {
            Member::Segment(segment) => parse_segment(&value, segment)?,
            Member::Index(index) => parse_index(&value, index)?,
            Member::Slice(slice) => parse_slice(&value, slice)?,
        }
    }

    Ok(value)
}

fn parse_segment(value: &SolValue, segment: &Segment) -> Result<SolValue, ParseError> {
    match value {
        SolValue::Tuple(entries) => {
            if let Ok(index) = segment.0.parse::<usize>() {
                Ok(entries
                    .get(index)
                    .ok_or_else(|| {
                        ParseError::ReferenceNotFound(alloc::format!(
                            "Parameter index {} out of bounds",
                            index
                        ))
                    })?
                    .1
                    .clone())
            } else {
                entries
                    .iter()
                    .find(|(name, _)| name.as_deref() == Some(segment.0.as_str()))
                    .map(|(_, val)| val.clone())
                    .ok_or_else(|| {
                        ParseError::ReferenceNotFound(alloc::format!(
                            "Field '{}' not found in tuple",
                            segment.0
                        ))
                    })
            }
        }
        _ => Err(ParseError::SmthWentWrong(alloc::format!(
            "Invalid value type for field access: {}",
            segment.0
        ))),
    }
}

fn parse_index(value: &SolValue, index: &Index) -> Result<SolValue, ParseError> {
    match value {
        SolValue::Bytes(bytes) => {
            let index = get_index(index.0, bytes.len())?;
            let byte = bytes[index];
            Ok(SolValue::Uint(U256::from(byte), 8))
        }
        SolValue::String(chars) => {
            let index = get_index(index.0, chars.len())?;
            let chars = &chars[index..index + 1];
            Ok(SolValue::String(chars.to_string()))
        }
        SolValue::Array(values) => {
            let index = get_index(index.0, values.len())?;
            Ok(values[index].clone())
        }
        SolValue::FixedArray(values) => {
            let index = get_index(index.0, values.len())?;
            Ok(values[index].clone())
        }
        _ => Err(ParseError::SmthWentWrong(alloc::format!(
            "Cannot index into type {}: only arrays, bytes, and strings support indexing",
            index.0
        ))),
    }
}

fn parse_slice(value: &SolValue, slice: &Slice) -> Result<SolValue, ParseError> {
    match value {
        SolValue::Bytes(bytes) => {
            let len = bytes.len();
            let start = get_index(slice.0.unwrap_or(0), len)?;
            let end = get_index(slice.1.unwrap_or(len.cast_signed()) - 1, len)?;
            let bytes = &bytes[start..=end];
            Ok(SolValue::Bytes(bytes.to_vec()))
        }
        SolValue::String(chars) => {
            let len = chars.len();
            let start = get_index(slice.0.unwrap_or(0), len)?;
            let end = get_index(slice.1.unwrap_or(len.cast_signed()) - 1, len)?;
            let chars = &chars[start..=end];
            Ok(SolValue::String(chars.to_string()))
        }
        SolValue::Array(values) => {
            let len = values.len();
            let start = get_index(slice.0.unwrap_or(0), len)?;
            let end = get_index(slice.1.unwrap_or(len.cast_signed()) - 1, len)?;
            Ok(SolValue::Array(values[start..=end].to_vec()))
        }
        SolValue::FixedArray(values) => {
            let len = values.len();
            let start = get_index(slice.0.unwrap_or(0), len)?;
            let end = get_index(slice.1.unwrap_or(len.cast_signed()) - 1, len)?;
            Ok(SolValue::Array(values[start..=end].to_vec()))
        }
        _ => Err(ParseError::SmthWentWrong(alloc::format!(
            "Cannot slice type {:?} {:?}: only arrays, bytes, and strings support slicing",
            slice.0,
            slice.1,
        ))),
    }
}

fn get_index(index: isize, len: usize) -> Result<usize, ParseError> {
    if index >= 0 {
        let idx = index.cast_unsigned();
        if idx >= len {
            Err(ParseError::SmthWentWrong(alloc::format!(
                "Index {} out of bounds for length {}",
                index,
                len
            )))
        } else {
            Ok(idx)
        }
    } else {
        let idx = index.abs().cast_unsigned();
        if len >= idx {
            Ok(len - idx)
        } else {
            Err(ParseError::SmthWentWrong(alloc::format!(
                "Index {} out of bounds for length {}",
                index,
                len
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloy_primitives::{address, bytes, uint};

    static SENDER: Address = address!("0000000000000000000000000000000000000001");
    static TARGET: Address = address!("0000000000000000000000000000000000000002");
    static VALUE: U256 = uint!(1000000000000000000_U256);
    static DATA: Bytes = bytes!("0x12345678");

    fn create_test_context() -> (Message, SolValue) {
        let message = Message::new(SENDER, TARGET, VALUE, DATA.clone());

        let data = SolValue::Tuple(vec![
            (Some("amount".into()), SolValue::Uint(uint!(42_U256), 256)),
            (
                Some("recipient".into()),
                SolValue::Address(address!("0000000000000000000000000000000000000003")),
            ),
            (
                Some("message".into()),
                SolValue::String("Hello World".into()),
            ),
            (
                Some("items".into()),
                SolValue::Array(vec![
                    SolValue::Uint(uint!(10_U256), 256),
                    SolValue::Uint(uint!(20_U256), 256),
                    SolValue::Uint(uint!(30_U256), 256),
                ]),
            ),
            (
                Some("data".into()),
                SolValue::Bytes(vec![0xaa, 0xbb, 0xcc, 0xdd]),
            ),
            (
                Some("tup".into()),
                SolValue::Tuple(vec![(
                    None,
                    SolValue::Address(address!("0000000000000000000000000000000000000004")),
                )]),
            ),
            (
                Some("tuples".into()),
                SolValue::Array(vec![
                    SolValue::Tuple(vec![(
                        None,
                        SolValue::Array(vec![SolValue::Address(address!(
                            "0000000000000000000000000000000000000005"
                        ))]),
                    )]),
                    SolValue::Tuple(vec![(
                        None,
                        SolValue::Array(vec![SolValue::Address(address!(
                            "0000000000000000000000000000000000000006"
                        ))]),
                    )]),
                    SolValue::Tuple(vec![(
                        None,
                        SolValue::Array(vec![SolValue::Address(address!(
                            "0000000000000000000000000000000000000007"
                        ))]),
                    )]),
                ]),
            ),
        ]);

        (message, data)
    }

    #[test]
    fn test_resolve_msg_sender() {
        let (message, data) = create_test_context();
        let result = resolve_value("$msg.sender", &message, &data).unwrap();
        assert_eq!(result, SolValue::Address(SENDER));
    }

    #[test]
    fn test_resolve_msg_to() {
        let (message, data) = create_test_context();
        let result = resolve_value("$msg.to", &message, &data).unwrap();
        assert_eq!(result, SolValue::Address(TARGET));
    }

    #[test]
    fn test_resolve_msg_value() {
        let (message, data) = create_test_context();
        let result = resolve_value("$msg.value", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(VALUE, 256));
    }

    #[test]
    fn test_resolve_msg_data() {
        let (message, data) = create_test_context();
        let result = resolve_value("$msg.data", &message, &data).unwrap();
        assert_eq!(result, SolValue::Bytes(DATA.to_vec()));
    }

    #[test]
    fn test_resolve_msg_invalid_field() {
        let (message, data) = create_test_context();
        let result = resolve_value("$msg.invalid", &message, &data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::SmthWentWrong(_)));
    }

    #[test]
    fn test_resolve_param_by_name() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.amount", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(42_U256), 256));
    }

    #[test]
    fn test_resolve_param_by_index() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.0", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(42_U256), 256));

        let result = resolve_value("$data.1", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Address(address!("0000000000000000000000000000000000000003"))
        );
    }

    #[test]
    fn test_resolve_param_not_found() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.notfound", &message, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_array_index() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.items[0]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(10_U256), 256));

        let result = resolve_value("$data.items[1]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(20_U256), 256));

        let result = resolve_value("$data.items[2]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(30_U256), 256));
    }

    #[test]
    fn test_resolve_array_negative_index() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.items[-1]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(30_U256), 256));

        let result = resolve_value("$data.items[-2]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(20_U256), 256));

        let result = resolve_value("$data.items[-3]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(10_U256), 256));
    }

    #[test]
    fn test_resolve_array_out_of_bounds() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.items[99]", &message, &data);
        assert!(result.is_err());

        let result = resolve_value("$data.items[-99]", &message, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_string_index() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.message[0]", &message, &data).unwrap();
        assert_eq!(result, SolValue::String("H".into()));

        let result = resolve_value("$data.message[6]", &message, &data).unwrap();
        assert_eq!(result, SolValue::String("W".into()));
    }

    #[test]
    fn test_resolve_string_negative_index() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.message[-1]", &message, &data).unwrap();
        assert_eq!(result, SolValue::String("d".into()));
    }

    #[test]
    fn test_resolve_bytes_index() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.data[0]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(0xaa_U256), 8));

        let result = resolve_value("$data.data[1]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Uint(uint!(0xbb_U256), 8));
    }

    #[test]
    fn test_resolve_array_slice() {
        let (message, data) = create_test_context();

        // Full slice
        let result = resolve_value("$data.items[:]", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Array(vec![
                SolValue::Uint(uint!(10_U256), 256),
                SolValue::Uint(uint!(20_U256), 256),
                SolValue::Uint(uint!(30_U256), 256),
            ])
        );

        // Slice from start
        let result = resolve_value("$data.items[1:]", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Array(vec![
                SolValue::Uint(uint!(20_U256), 256),
                SolValue::Uint(uint!(30_U256), 256),
            ])
        );

        // Slice to end
        let result = resolve_value("$data.items[:2]", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Array(vec![
                SolValue::Uint(uint!(10_U256), 256),
                SolValue::Uint(uint!(20_U256), 256),
            ])
        );

        // Slice middle
        let result = resolve_value("$data.items[1:2]", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Array(vec![SolValue::Uint(uint!(20_U256), 256),])
        );
    }

    #[test]
    fn test_resolve_string_slice() {
        let (message, data) = create_test_context();

        let result = resolve_value("$data.message[0:5]", &message, &data).unwrap();
        assert_eq!(result, SolValue::String("Hello".into()));

        let result = resolve_value("$data.message[6:]", &message, &data).unwrap();
        assert_eq!(result, SolValue::String("World".into()));

        let result = resolve_value("$data.message[:5]", &message, &data).unwrap();
        assert_eq!(result, SolValue::String("Hello".into()));
    }

    #[test]
    fn test_resolve_bytes_slice() {
        let (message, data) = create_test_context();

        let result = resolve_value("$data.data[1:3]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Bytes(vec![0xbb, 0xcc]));

        let result = resolve_value("$data.data[:2]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Bytes(vec![0xaa, 0xbb]));

        let result = resolve_value("$data.data[2:]", &message, &data).unwrap();
        assert_eq!(result, SolValue::Bytes(vec![0xcc, 0xdd]));
    }

    #[test]
    fn test_resolve_slice_with_negative_indices() {
        let (message, data) = create_test_context();

        let result = resolve_value("$data.items[-2:]", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Array(vec![
                SolValue::Uint(uint!(20_U256), 256),
                SolValue::Uint(uint!(30_U256), 256),
            ])
        );

        let result = resolve_value("$data.message[:-6]", &message, &data).unwrap();
        assert_eq!(result, SolValue::String("Hello".into()));
    }

    #[test]
    fn test_resolve_invalid_path() {
        let (message, data) = create_test_context();

        // Invalid prefix
        let result = resolve_value("$invalid.field", &message, &data);
        assert!(result.is_err());

        // Missing path
        let result = resolve_value("$msg", &message, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_type_mismatch() {
        let (message, data) = create_test_context();

        // Can't index into uint256
        let result = resolve_value("$data.amount[0]", &message, &data);
        assert!(result.is_err());

        // Can't slice uint256
        let result = resolve_value("$data.amount[1:2]", &message, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_tuple_field() {
        let (message, data) = create_test_context();
        let result = resolve_value("$data.tup.0", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Address(address!("0000000000000000000000000000000000000004"))
        );
    }

    #[test]
    fn test_resolve_tuple_array_field() {
        let (message, data) = create_test_context();

        let result = resolve_value("$data.tuples[0].0[0]", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Address(address!("0000000000000000000000000000000000000005"))
        );

        let result = resolve_value("$data.tuples[1].0[0]", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Address(address!("0000000000000000000000000000000000000006"))
        );

        let result = resolve_value("$data.tuples[2].0[0]", &message, &data).unwrap();
        assert_eq!(
            result,
            SolValue::Address(address!("0000000000000000000000000000000000000007"))
        );
    }

    #[test]
    fn test_resolve_literal() {
        let (message, data) = create_test_context();
        let result = resolve_value("Simple Literal", &message, &data).unwrap();
        assert_eq!(result, SolValue::Literal("Simple Literal".into()));
    }
}

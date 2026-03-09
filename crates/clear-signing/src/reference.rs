use crate::ResultExt;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use winnow::{ascii::digit1, combinator::{alt, delimited, opt, preceded, repeat}, token::take_while, ModalResult, Parser};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use winnow::error::ContextError;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum Reference {
    Identifier {
        identifier: Identifier,
        reference: String,
    },
    Literal(String),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub container: String,
    pub members: Vec<Member>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct Segment(pub String);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct Index(pub isize);
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct Slice(pub Option<isize>, pub Option<isize>);
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum Member {
    Segment(Segment),
    Index(Index),
    Slice(Slice),
}

impl Member {
    fn index(index: isize) -> Member {
        Member::Index(Index(index))
    }

    fn segment(name: &str) -> Member {
        Member::Segment(Segment(name.into()))
    }

    fn slice(start: Option<isize>, end: Option<isize>) -> Member {
        Member::Slice(Slice(start, end))
    }
}

impl Reference {
    pub fn parse(path: &str) -> crate::Result<Self> {
        if path.is_empty() {
            return Ok(Reference::Literal("".into()));
        }

        let parsed = parse_value.parse(path)
            .err_ctx("Winnow parse error")?;
        Ok(parsed)
    }
}

fn parse_identifier<'s>(input: &mut &'s str) -> ModalResult<&'s str, ContextError> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_')
        .parse_next(input)
}

fn parse_reference(input: &mut &str) -> ModalResult<Reference, ContextError> {
    let start = *input;
    let container = preceded('$', parse_identifier).parse_next(input)?;
    let members = repeat(0.., alt((parse_bracket, preceded('.', parse_segment))))
        .parse_next(input)?;

    Ok(Reference::Identifier {
        identifier: Identifier { container: container.to_string(), members },
        reference: start.to_string(),
    })
}

fn parse_literal(input: &mut &str) -> ModalResult<Reference, ContextError> {
    take_while(1.., |c: char| c.is_ascii())
        .verify(|s: &str| !s.starts_with('$'))
        .map(|s: &str| Reference::Literal(s.to_string()))
        .parse_next(input)
}

fn parse_value(input: &mut &str) -> ModalResult<Reference, ContextError> {
    alt((parse_reference, parse_literal)).parse_next(input)
}

fn parse_segment(input: &mut &str) -> ModalResult<Member, ContextError> {
    let s = parse_identifier.parse_next(input)?;
    Ok(Member::segment(s))
}

fn parse_isize(input: &mut &str) -> ModalResult<isize, ContextError> {
    (opt('-'), digit1)
        .take()
        .try_map(|s: &str| s.parse::<isize>())
        .parse_next(input)
}

fn parse_index_inner(input: &mut &str) -> ModalResult<Member, ContextError> {
    let idx = parse_isize.parse_next(input)?;
    Ok(Member::index(idx))
}

fn parse_slice_inner(input: &mut &str) -> ModalResult<Member, ContextError> {
    let (start, _, end) = (opt(parse_isize), ':', opt(parse_isize)).parse_next(input)?;
    Ok(Member::slice(start, end))
}

fn parse_bracket(input: &mut &str) -> ModalResult<Member, ContextError> {
    delimited('[', alt((parse_slice_inner, parse_index_inner)), ']').parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_param() {
        assert!(Reference::parse("$").is_err());
        assert!(Reference::parse("$par@m").is_err());
        assert!(Reference::parse("$par,m").is_err());
        assert!(Reference::parse("$param.").is_err());
        assert!(Reference::parse("$param..field").is_err());

        assert_eq!(
            Reference::parse("hello world").unwrap(),
            Reference::Literal("hello world".into())
        );
        assert!(Reference::parse("🌟 emoji").is_err());

        match Reference::parse("$param").unwrap() {
            Reference::Identifier {
                identifier,
                reference,
            } => {
                assert_eq!(identifier.container, "param");
                assert!(identifier.members.is_empty());
                assert_eq!(reference, "$param");
            }
            _ => panic!("Expected Reference"),
        }

        match Reference::parse("$param.field").unwrap() {
            Reference::Identifier {
                identifier,
                reference,
            } => {
                assert_eq!(identifier.container, "param");
                assert_eq!(identifier.members, vec![Member::segment("field")]);
                assert_eq!(reference, "$param.field");
            }
            _ => panic!("Expected Reference"),
        }
    }

    #[test]
    fn test_index() {
        assert!(Reference::parse("$param[]").is_err());

        match Reference::parse("$param[1]").unwrap() {
            Reference::Identifier {
                identifier,
                reference: _,
            } => {
                assert_eq!(identifier.container, "param");
                assert_eq!(identifier.members, vec![Member::index(1)]);
            }
            _ => panic!("Expected Reference"),
        }

        match Reference::parse("$param[-1]").unwrap() {
            Reference::Identifier {
                identifier,
                reference: _,
            } => {
                assert_eq!(identifier.container, "param");
                assert_eq!(identifier.members, vec![Member::index(-1)]);
            }
            _ => panic!("Expected Reference"),
        }
    }

    #[test]
    fn test_slice() {
        assert!(Reference::parse("$param[::]").is_err());

        match Reference::parse("$param[:]").unwrap() {
            Reference::Identifier {
                identifier,
                reference: _,
            } => {
                assert_eq!(identifier.container, "param");
                assert_eq!(identifier.members, vec![Member::slice(None, None)]);
            }
            _ => panic!("Expected Reference"),
        }

        match Reference::parse("$param[1:]").unwrap() {
            Reference::Identifier {
                identifier,
                reference: _,
            } => {
                assert_eq!(identifier.container, "param");
                assert_eq!(identifier.members, vec![Member::slice(Some(1), None)]);
            }
            _ => panic!("Expected Reference"),
        }

        match Reference::parse("$param[:1]").unwrap() {
            Reference::Identifier {
                identifier,
                reference: _,
            } => {
                assert_eq!(identifier.container, "param");
                assert_eq!(identifier.members, vec![Member::slice(None, Some(1))]);
            }
            _ => panic!("Expected Reference"),
        }

        match Reference::parse("$param[1:10]").unwrap() {
            Reference::Identifier {
                identifier,
                reference: _,
            } => {
                assert_eq!(identifier.container, "param");
                assert_eq!(identifier.members, vec![Member::slice(Some(1), Some(10))]);
            }
            _ => panic!("Expected Reference"),
        }
    }

    #[test]
    fn test_path() {
        match Reference::parse("$path.to[4][:6][2:][5:10]").unwrap() {
            Reference::Identifier {
                identifier,
                reference: _,
            } => {
                assert_eq!(identifier.container, "path");
                assert_eq!(
                    identifier.members,
                    vec![
                        Member::segment("to"),
                        Member::index(4),
                        Member::slice(None, Some(6)),
                        Member::slice(Some(2), None),
                        Member::slice(Some(5), Some(10)),
                    ]
                );
            }
            _ => panic!("Expected Reference"),
        }
    }
}

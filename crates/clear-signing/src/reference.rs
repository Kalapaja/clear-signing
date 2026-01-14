use crate::error::ParseError;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use nom::combinator::{all_consuming, map, map_res, verify};
use nom::multi::many0;
use nom::{
    branch::alt, bytes::complete::take_while1,
    character::complete::{char, digit1},
    combinator::{opt, recognize},
    sequence::{delimited, preceded},
    IResult,
    Parser,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Reference {
    Identifier {
        identifier: Identifier,
        reference: String,
    },
    Literal(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub container: String,
    pub members: Vec<Member>,
}

impl Identifier {
    pub fn only_segments(&self) -> Result<String, ParseError> {
        let mut result = String::new();

        for member in &self.members {
            match member {
                Member::Segment(seg) => result.push_str(&seg.0),
                Member::Index(_) => {
                    return Err(ParseError::SmthWentWrong("Expected segment member".into()));
                }
                Member::Slice(_) => {
                    return Err(ParseError::SmthWentWrong("Expected segment member".into()));
                }
            };
        }

        Ok(result)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Segment(pub String);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Index(pub isize);
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Slice(pub Option<isize>, pub Option<isize>);
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
    pub fn parse(path: &str) -> Result<Self, ParseError> {
        if path.is_empty() {
            return Ok(Reference::Literal("".into()));
        }

        let parsed = all_consuming(parse_value).parse(path)?;
        Ok(parsed.1)
    }
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_'),
        |s: &str| s.to_string(),
    )
    .parse(input)
}

fn parse_reference(input: &str) -> IResult<&str, Reference> {
    all_consuming(map(
        (
            preceded(char('$'), parse_identifier),
            many0(alt((parse_bracket, preceded(char('.'), parse_segment)))),
        ),
        |(container, members)| Reference::Identifier {
            identifier: Identifier { container, members },
            reference: input.to_string(),
        },
    ))
    .parse(input)
}

fn parse_literal(input: &str) -> IResult<&str, Reference> {
    map(
        verify(take_while1(|c: char| c.is_ascii()), |s: &str| {
            !s.starts_with('$')
        }),
        |s: &str| Reference::Literal(s.to_string()),
    )
    .parse(input)
}

fn parse_value(input: &str) -> IResult<&str, Reference> {
    alt((parse_reference, parse_literal)).parse(input)
}

fn parse_segment(input: &str) -> IResult<&str, Member> {
    map(parse_identifier, |s| Member::segment(&s)).parse(input)
}

fn parse_isize(input: &str) -> IResult<&str, isize> {
    map_res(recognize((opt(char('-')), digit1)), |s: &str| {
        s.parse::<isize>()
    })
    .parse(input)
}

fn parse_index_inner(input: &str) -> IResult<&str, Member> {
    map(parse_isize, Member::index).parse(input)
}

fn parse_slice_inner(input: &str) -> IResult<&str, Member> {
    let (input, (start, _, end)) = (opt(parse_isize), char(':'), opt(parse_isize)).parse(input)?;

    Ok((input, Member::slice(start, end)))
}

fn parse_bracket(input: &str) -> IResult<&str, Member> {
    delimited(
        char('['),
        alt((parse_slice_inner, parse_index_inner)),
        char(']'),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_param() {
        // Invalid references are now Errors (Literal cannot start with $)
        assert!(Reference::parse("$").is_err());
        assert!(Reference::parse("$par@m").is_err());
        assert!(Reference::parse("$par,m").is_err());
        assert!(Reference::parse("$param.").is_err());
        assert!(Reference::parse("$param..field").is_err());

        // Literals can be anything ASCII (except starting with $)
        assert_eq!(
            Reference::parse("hello world").unwrap(),
            Reference::Literal("hello world".into())
        );
        // Non-ASCII literals should fail
        assert!(Reference::parse("🌟 emoji").is_err());

        // Valid references
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
        // Invalid reference starting with $ is Error
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

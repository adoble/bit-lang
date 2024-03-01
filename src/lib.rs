#![allow(dead_code)]
pub mod parser;
pub use parser::{BitRange, BitSpec, Condition, Repeat, Word};

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    ParseError,
}

fn parse(bit_spec_string: &str) -> Result<BitSpec, Error> {
    let (_, bit_spec) = parser::bit_spec(bit_spec_string).map_err(|_| Error::ParseError)?;

    Ok(bit_spec)
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_bit_spec_with_simple_forms() {
        let data = "4";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Single(4),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "4..6";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Range(4, 6),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "[4..6]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::Range(4, 6),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "5[3..7]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 5,
                bit_range: BitRange::Range(3, 7),
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "5[]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);
    }

    #[test]
    fn test_bit_spec_with_repeat() {
        let data = "3[4..7]..6[0..5];48";

        let bit_spec = parse(data).unwrap();

        let expected = BitSpec {
            start: Word {
                index: 3,
                bit_range: BitRange::Range(4, 7),
            },
            end: Some(Word {
                index: 6,
                bit_range: BitRange::Range(0, 5),
            }),
            repeat: Repeat::Fixed(48),
        };
        assert_eq!(bit_spec, expected);

        let data = "4[]..7[];(3[])<49";
        let bit_spec = parse(data).unwrap();
        let repeat = Repeat::Variable {
            word: Word {
                index: 3,
                bit_range: BitRange::WholeWord,
            },
            condition: Condition::Lt,
            limit: 49,
        };
        let expected = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 7,
                bit_range: BitRange::WholeWord,
            }),
            repeat: repeat,
        };
        assert_eq!(bit_spec, expected);
    }
    #[test]
    fn test_bit_spec() {
        let data = "3[4..7]..6[0..5]";

        let bit_spec = parse(data).unwrap();

        let expected = BitSpec {
            start: Word {
                index: 3,
                bit_range: BitRange::Range(4, 7),
            },
            end: Some(Word {
                index: 6,
                bit_range: BitRange::Range(0, 5),
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "4[]..7[]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 7,
                bit_range: BitRange::WholeWord,
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "[]..5[]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "[]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: None,
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);

        let data = "[]..6[0..5]";
        let bit_spec = parse(data).unwrap();
        let expected = BitSpec {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 6,
                bit_range: BitRange::Range(0, 5),
            }),
            repeat: Repeat::None,
        };
        assert_eq!(bit_spec, expected);
    }
}

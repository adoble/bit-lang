use nom::combinator::all_consuming;
#[allow(unused_imports)]
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take_while1},
    character::complete::u8 as u8_parser,
    character::complete::{char, one_of, space0},
    character::is_digit,
    combinator::{into, map, opt, recognize, value},
    multi::{many0, many1},
    //number::complete::{i32, u8},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
    Parser,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BitRange {
    Single(u8),
    Range(u8, u8),
    WholeWord,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Word {
    // No index refers to index = 0
    index: u8,
    // No bit spec refers to the whole word
    bit_range: BitRange,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Condition {
    Lt,
    Lte,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Repeat {
    // A simple fixed number of repititions
    Fixed(u8),
    // A variable number of repitions determined by another word and limited
    Variable {
        word: Word,
        condition: Condition,
        limit: u8,
    },
}

// impl From<u8> for Limit {
//     fn from(value: u8) -> Self {
//         Limit::Literal(value)
//     }
// }

// impl From<Word> for Limit {
//     fn from(word: Word) -> Self {
//         Limit::Word(word)
//     }
// }

#[derive(Debug, PartialEq, Copy, Clone)]
struct WordRange {
    start: Word,
    end: Option<Word>,
    //repeat: Option<Repeat>,
}

fn index(input: &str) -> IResult<&str, u8> {
    (u8_parser)(input)
}

fn single_bit(input: &str) -> IResult<&str, BitRange> {
    let (remaining, position) = (index)(input)?;

    Ok((remaining, BitRange::Single(position)))
}

fn range(input: &str) -> IResult<&str, BitRange> {
    //tuple((index, tag(".."), index))(input)
    let (remaining, (start, stop)) = separated_pair(index, tag(".."), index)(input)?;

    Ok((remaining, BitRange::Range(start, stop)))
}

fn bit_range(input: &str) -> IResult<&str, BitRange> {
    alt((range, single_bit))(input) // Order importantre
}

fn full_word(input: &str) -> IResult<&str, Word> {
    let (remaining, (index, _, bit_range, _)) =
        tuple((opt(index), tag("["), opt(bit_range), tag("]")))(input)?;

    let completed_bit_range = match bit_range {
        Some(bit_range) => bit_range,
        None => BitRange::WholeWord,
    };

    Ok((
        remaining,
        Word {
            index: index.unwrap_or(0),
            bit_range: completed_bit_range,
        },
    ))
}

// A bit spec - e.g. "3" or "4..6"  is also treated as a full word, i.e.
// "0[3]" or "0[4,..6]" respectively. This function maps the bit spec to
// a word for later inclusion in highe level parsers
fn bit_range_as_word(input: &str) -> IResult<&str, Word> {
    let (remaining, bit_range) = (bit_range)(input)?;

    Ok((
        remaining,
        Word {
            index: 0,
            bit_range,
        },
    ))
}

fn word_literal(input: &str) -> IResult<&str, Word> {
    let (input, index) = opt(index)(input)?;
    let (input, _) = tag("[")(input)?;
    let (input, literal) = literal(input)?;
    let (remaining, _) = tag("]")(input)?;

    todo!()

    // Ok((
    //     remaining,
    //     Word {
    //         index: 0,
    //         bit_range,
    //     },
    // ))
}

// word = bit_range | [index] "[" [bit_range] "]" | index "[" literal "]";   (* NEW *)
// TODO Ignore literals for the time being
fn word(input: &str) -> IResult<&str, Word> {
    let (remaining, word) = alt((full_word, bit_range_as_word))(input)?;

    Ok((remaining, word))
}

// word_range = word [".." word] [repeat]
// TODO ignore repeats for now
fn word_range(input: &str) -> IResult<&str, WordRange> {
    let (remaining, (start, end_option)) = tuple((word, opt(tuple((tag(".."), word)))))(input)?;

    let end = match end_option {
        Some((_, w)) => Some(w),
        None => None,
    };

    Ok((remaining, WordRange { start, end }))
}

fn condition(input: &str) -> IResult<&str, Condition> {
    let (remaining, condition) = alt((
        value(Condition::Lte, tag("<=")),
        value(Condition::Lt, tag("<")),
    ))(input)?;

    Ok((remaining, condition))
}

// fn fixed_limit_parser(input: &str) -> IResult<&str, Limit> {
//     let (remaining, limit) = (u8_parser)(input)?;

//     Ok((remaining, Limit::Literal(limit)))
// }

// fn word_limit_parser(input: &str) -> IResult<&str, Limit> {
//     let (remaining, limit) = (full_word)(input)?;

//     Ok((remaining, Limit::Word(limit)))
// }

// fn limit(input: &str) -> IResult<&str, Limit> {
//     let (remaining, limit) = alt((word_limit_parser, fixed_limit_parser))(input)?;

//     Ok((remaining, limit))
// }

fn fixed_repeat(input: &str) -> IResult<&str, Repeat> {
    // let (remaining, r) = (u8_parser.map(|value| Repeat::Fixed(value)))(input)?;
    let (remaining, repeat) = map(u8_parser, |value| Repeat::Fixed(value))(input)?;

    Ok((remaining, repeat))
}

// variable_word = "(" word ")";
fn variable_word(input: &str) -> IResult<&str, Word> {
    // TODO see if  we can also use take_until() to solve ambiguity
    let (remaining, word) = delimited(char('('), word, char(')'))(input)?;
    Ok((remaining, word))
}

// variable_repeat = variable_word condition limit;
fn variable_repeat(input: &str) -> IResult<&str, Repeat> {
    let (remaining, (word, condition, limit)) =
        tuple((variable_word, condition, u8_parser))(input)?;
    Ok((
        remaining,
        Repeat::Variable {
            word,
            condition,
            limit,
        },
    ))
}

// repeat = ";" (fixed_repeat  | variable_repeat)  ;
fn repeat(input: &str) -> IResult<&str, Repeat> {
    //let (remaining, (_, repeat)) = tuple((tag(";"), alt((variable_repeat, fixed_repeat))))(input)?;
    let (remaining, repeat) = preceded(tag(";"), alt((variable_repeat, fixed_repeat)))(input)?;

    Ok((remaining, repeat))
}

fn hexadecimal(input: &str) -> IResult<&str, &str> {
    // preceded(
    //     alt((tag("0x"), tag("0X"))),
    //     recognize(many1(terminated(
    //         one_of("0123456789abcdefABCDEF"),
    //         many0(char('_')),
    //     ))),
    // )
    // .parse(input)

    let (input, _) = alt((tag("0x"), tag("0X")))(input)?;
    //let (remaining, bin) = recognize(many1(one_of("01_")))(input)?;
    let (remaining, hex) =
        recognize(all_consuming(many1(one_of("0123456789abcdefABCDEF_"))))(input)?;

    Ok((remaining, hex))
}

fn binary(input: &str) -> IResult<&str, &str> {
    let (input, _) = alt((tag("0b"), tag("0B")))(input)?;
    //let (remaining, bin) = recognize(many1(one_of("01_")))(input)?;
    let (remaining, bin) = recognize(all_consuming(many1(one_of("01_"))))(input)?;

    Ok((remaining, bin))
}

#[allow(dead_code)]
fn literal(input: &str) -> IResult<&str, &str> {
    let (remaining, literal) = alt((hexadecimal, binary))(input)?;
    Ok((remaining, literal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat() {
        let data = ";12";
        let (_, r) = repeat(data).unwrap();
        assert_eq!(r, Repeat::Fixed(12));

        let data = ";6";
        let (_, r) = repeat(data).unwrap();
        assert_eq!(r, Repeat::Fixed(6));

        let data = ";(4[])<49";
        let (_, r) = repeat(data).unwrap();
        let word = Word {
            index: 4,
            bit_range: BitRange::WholeWord,
        };

        let expected = Repeat::Variable {
            word,
            condition: Condition::Lt,
            limit: 49,
        };
        assert_eq!(r, expected);

        let data = ";(4[])";
        assert!(repeat(data).is_err());
    }

    #[test]
    fn test_variable_repeat() {
        let data = "(4[])<=48";
        let (_, r) = variable_repeat(data).unwrap();
        let word = Word {
            index: 4,
            bit_range: BitRange::WholeWord,
        };

        let expected = Repeat::Variable {
            word,
            condition: Condition::Lte,
            limit: 48,
        };
        assert_eq!(r, expected);

        let data = "(4[0..7])<49";
        let (_, r) = variable_repeat(data).unwrap();
        let word = Word {
            index: 4,
            bit_range: BitRange::Range(0, 7),
        };

        let expected = Repeat::Variable {
            word,
            condition: Condition::Lt,
            limit: 49,
        };
        assert_eq!(r, expected);
    }
    #[test]
    fn test_fixed_repeat() {
        let data = "48";
        let (_, r) = fixed_repeat(data).unwrap();
        assert_eq!(r, Repeat::Fixed(48));
    }

    #[test]
    fn test_condition() {
        let data = "<";
        let (_, r) = condition(data).unwrap();
        assert_eq!(r, Condition::Lt);

        let data = "<=";
        let (_, r) = condition(data).unwrap();
        assert_eq!(r, Condition::Lte);

        let data = "";
        assert!(condition(data).is_err());

        let data = "==";
        assert!(condition(data).is_err());
    }

    #[test]
    fn test_word_range() {
        let data = "3[4..7]..6[0..5]";

        let (_, r) = word_range(data).unwrap();

        let expected = WordRange {
            start: Word {
                index: 3,
                bit_range: BitRange::Range(4, 7),
            },
            end: Some(Word {
                index: 6,
                bit_range: BitRange::Range(0, 5),
            }),
        };
        assert_eq!(r, expected);

        let data = "4[]..7[]";
        let (_, r) = word_range(data).unwrap();
        let expected = WordRange {
            start: Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 7,
                bit_range: BitRange::WholeWord,
            }),
        };
        assert_eq!(r, expected);

        let data = "[]..5[]";
        let (_, r) = word_range(data).unwrap();
        let expected = WordRange {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 5,
                bit_range: BitRange::WholeWord,
            }),
        };
        assert_eq!(r, expected);

        let data = "[]";
        let (_, r) = word_range(data).unwrap();
        let expected = WordRange {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: None,
        };
        assert_eq!(r, expected);

        let data = "[]..6[0..5]";
        let (_, r) = word_range(data).unwrap();
        let expected = WordRange {
            start: Word {
                index: 0,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: 6,
                bit_range: BitRange::Range(0, 5),
            }),
        };
        assert_eq!(r, expected);
    }

    // Not recommand, but still accepted patterns.
    #[test]
    fn test_word_range_special_cases() {
        let data = "3..5..4[]";
        let (_, r) = word_range(data).unwrap();
        let expected = WordRange {
            start: Word {
                index: 0,
                bit_range: BitRange::Range(3, 5),
            },
            end: Some(Word {
                index: 4,
                bit_range: BitRange::WholeWord,
            }),
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn test_word() {
        let data = "3[2..6]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 3,
                bit_range: BitRange::Range(2, 6)
            }
        );

        let data = "4[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 4,
                bit_range: BitRange::WholeWord
            }
        );

        let data = "[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 0,
                bit_range: BitRange::WholeWord
            }
        );

        let data = "3[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 3,
                bit_range: BitRange::WholeWord
            }
        );

        let data = "7";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 0,
                bit_range: BitRange::Single(7)
            }
        )
    }

    #[test]
    fn test_full_word() {
        let data = "3[4..6]";
        let (_, r) = full_word(data).unwrap();

        let expected_word = Word {
            index: 3,
            bit_range: BitRange::Range(4, 6),
        };

        assert_eq!(r, expected_word);

        let data = "9[]";
        let (_, r) = full_word(data).unwrap();
        let expected_word = Word {
            index: 9,
            bit_range: BitRange::WholeWord,
        };
        assert_eq!(r, expected_word);

        let data = "[3..7]";
        let (_, r) = full_word(data).unwrap();
        let expected_word = Word {
            index: 0,
            bit_range: BitRange::Range(3, 7),
        };
        assert_eq!(r, expected_word);

        let data = "[]";
        let (_, r) = full_word(data).unwrap();
        let expected_word = Word {
            index: 0,
            bit_range: BitRange::WholeWord,
        };
        assert_eq!(r, expected_word);
    }

    #[test]
    fn test_bit_range() {
        let data = "4..6";
        let (_, r) = bit_range(data).unwrap();
        assert_eq!(r, BitRange::Range(4, 6));

        let data = "7";
        let (_, r) = bit_range(data).unwrap();
        assert_eq!(r, BitRange::Single(7));
    }

    #[test]
    fn test_single_bit() {
        let data = "2";
        let (_, r) = single_bit(data).unwrap();
        assert_eq!(r, BitRange::Single(2));
    }

    #[test]
    fn test_range() {
        let data = "2..45";
        let (_, r) = range(data).unwrap();
        assert_eq!(r, BitRange::Range(2, 45));
    }

    #[test]
    fn test_index() {
        let data = "34";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, 34);

        let data = "7";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, 7);

        let data = "48";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, 48);
    }

    #[test]
    fn test_literal() {
        let data = "0xABCD";
        let (_, r) = literal(data).unwrap();
        assert_eq!(r, "ABCD");

        let data = "0b1011_1100";
        let (_, r) = literal(data).unwrap();
        assert_eq!(r, "1011_1100");

        let data = "0b1011_11b0";
        assert!(literal(data).is_err());

        let data = "0Xab_zc";
        assert!(literal(data).is_err());
    }

    #[test]
    fn test_hexadecimal() {
        let data = "0x45B7";
        let (_, hex) = hexadecimal(data).unwrap();
        assert_eq!(hex, "45B7");

        let data = "0X45_B7";
        let (_, hex) = hexadecimal(data).unwrap();
        assert_eq!(hex, "45_B7");

        // TODO test error
    }

    #[test]
    fn test_binary() {
        let data = "0b10001100";
        let (_, bin) = binary(data).unwrap();
        assert_eq!(bin, "10001100");

        let data = "0b1000_1100";
        let (_, bin) = binary(data).unwrap();
        assert_eq!(bin, "1000_1100");

        let data = "0b1100_AB";
        assert!(binary(data).is_err());
        // let (_, bin) = binary(data).unwrap();
        // assert_eq!(bin, "1000_1100");
    }
}

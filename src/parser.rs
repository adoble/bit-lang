#[allow(unused_imports)]
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take_while1},
    character::complete::{char, one_of, space0, u8},
    character::is_digit,
    combinator::{into, map, opt, recognize, value},
    multi::{many0, many1},
    //number::complete::{i32, u8},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
    Parser,
};

//TODO Change this to BitRange (as opposed to WordRange)
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BitRange {
    Single(u8),
    Range(u8, u8),
    WholeWord,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Word {
    // No index refers to index = 0
    index: Option<u8>,
    // No bit spec refers to the whole word
    bit_range: BitRange,
}

pub struct Repeat;

#[derive(Debug, PartialEq, Copy, Clone)]
struct WordRange {
    start: Word,
    end: Option<Word>,
    //repeat: Option<Repeat>,
}

fn index(input: &str) -> IResult<&str, u8> {
    (u8)(input)
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
            index,
            bit_range: completed_bit_range,
        },
    ))
}

// A bit spec - e.g. "3" or "4..6"  is also treaed as a full word, i.e.
// "0[3]" or "0[4,..6]" respectively. This function maps the bit spec to
// a word for later inclusion in highe level parsers
fn bit_range_as_word(input: &str) -> IResult<&str, Word> {
    let (remaining, bit_range) = (bit_range)(input)?;

    Ok((
        remaining,
        Word {
            index: None,
            bit_range,
        },
    ))
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

//fn pattern(input: &str) -> IResult<&str, Pattern> {}

fn hexadecimal(input: &str) -> IResult<&str, &str> {
    // <'a, E: ParseError<&'a str>>
    preceded(
        alt((tag("0x"), tag("0X"))),
        recognize(many1(terminated(
            one_of("0123456789abcdefABCDEF"),
            many0(char('_')),
        ))),
    )
    .parse(input)
}

// Supporting seperated literals such 0x34_AB
#[allow(dead_code)]
fn seperated_hexadecimal(_input: &str) -> IResult<&str, &str> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_range() {
        let data = "3[4..7]..6[0..5]";

        let (_, r) = word_range(data).unwrap();

        let expected = WordRange {
            start: Word {
                index: Some(3),
                bit_range: BitRange::Range(4, 7),
            },
            end: Some(Word {
                index: Some(6),
                bit_range: BitRange::Range(0, 5),
            }),
        };
        assert_eq!(r, expected);

        let data = "4[]..7[]";
        let (_, r) = word_range(data).unwrap();
        let expected = WordRange {
            start: Word {
                index: Some(4),
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: Some(7),
                bit_range: BitRange::WholeWord,
            }),
        };
        assert_eq!(r, expected);

        let data = "[]..5[]";
        let (_, r) = word_range(data).unwrap();
        let expected = WordRange {
            start: Word {
                index: None,
                bit_range: BitRange::WholeWord,
            },
            end: Some(Word {
                index: Some(5),
                bit_range: BitRange::WholeWord,
            }),
        };
        assert_eq!(r, expected);

        let data = "[]";
        let (_, r) = word_range(data).unwrap();
        let expected = WordRange {
            start: Word {
                index: None,
                bit_range: BitRange::WholeWord,
            },
            end: None,
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
                index: Some(3),
                bit_range: BitRange::Range(2, 6)
            }
        );

        let data = "4[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: Some(4),
                bit_range: BitRange::WholeWord
            }
        );

        let data = "[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: None,
                bit_range: BitRange::WholeWord
            }
        );

        let data = "3[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: Some(3),
                bit_range: BitRange::WholeWord
            }
        );

        let data = "7";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: None,
                bit_range: BitRange::Single(7)
            }
        )
    }

    #[test]
    fn test_full_word() {
        let data = "3[4..6]";
        let (_, r) = full_word(data).unwrap();

        let expected_word = Word {
            index: Some(3),
            bit_range: BitRange::Range(4, 6),
        };

        assert_eq!(r, expected_word);

        let data = "9[]";
        let (_, r) = full_word(data).unwrap();
        let expected_word = Word {
            index: Some(9),
            bit_range: BitRange::WholeWord,
        };
        assert_eq!(r, expected_word);

        let data = "[3..7]";
        let (_, r) = full_word(data).unwrap();
        let expected_word = Word {
            index: None,
            bit_range: BitRange::Range(3, 7),
        };
        assert_eq!(r, expected_word);

        let data = "[]";
        let (_, r) = full_word(data).unwrap();
        let expected_word = Word {
            index: None,
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
    fn test_hexadecimal() {
        let data = "0x45B7";
        let (_, hex) = hexadecimal(data).unwrap();

        assert_eq!(hex, "45B7");
    }
}

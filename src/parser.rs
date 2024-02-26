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
#[derive(Debug, PartialEq, Copy, Clone)]
enum BitSpec {
    Single(u8),
    Range(u8, u8),
    //WholeWord,
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Word {
    // No index refers to index = 0
    index: Option<u8>,
    // No bit spec refers to the whole word
    bit_spec: Option<BitSpec>,
}

struct Repeat;

struct Pattern {
    start: Word,
    end: Option<Word>,
    repeat: Option<Repeat>,
}

fn index(input: &str) -> IResult<&str, u8> {
    (u8)(input)
}

fn single_bit(input: &str) -> IResult<&str, BitSpec> {
    let (remaining, position) = (index)(input)?;

    Ok((remaining, BitSpec::Single(position)))
}

fn range(input: &str) -> IResult<&str, BitSpec> {
    //tuple((index, tag(".."), index))(input)
    let (remaining, (start, stop)) = separated_pair(index, tag(".."), index)(input)?;

    Ok((remaining, BitSpec::Range(start, stop)))
}

fn bit_spec(input: &str) -> IResult<&str, BitSpec> {
    alt((range, single_bit))(input) // Order importantre
}

// fn empty_bit_spec(input: &str) -> IResult<&str, BitSpec> {
//     // Required delimiters
//     let (remaining, _) = delimited(tag("["), space0, tag("]"))(input)?;
//     Ok((remaining, BitSpec::WholeWord))
// }

// fn bit_spec_delimited(input: &str) -> IResult<&str, BitSpec> {
//     let (remaining, bit_spec) =
//         alt((delimited(tag("["), bit_spec, tag("]")), empty_bit_spec))(input)?;
//     Ok((remaining, bit_spec))
// }

fn full_word(input: &str) -> IResult<&str, Word> {
    let (remaining, (index, _, bit_spec, _)) =
        tuple((opt(index), tag("["), opt(bit_spec), tag("]")))(input)?;

    Ok((remaining, Word { index, bit_spec }))
}

// A bit spec - e.g. "3" or "4..6"  is also treaed as a full word, i.e.
// "0[3]" or "0[4,..6]" respectively. This function maps the bit spec to
// a word for later inclusion in highe level parsers
fn bit_spec_as_word(input: &str) -> IResult<&str, Word> {
    let (remaining, bit_spec) = (bit_spec)(input)?;

    Ok((
        remaining,
        Word {
            index: None,
            bit_spec: Some(bit_spec),
        },
    ))
}

// word = bit_spec | [index] "[" [bit_spec] "]" | index "[" literal "]";   (* NEW *)
// TODO Ignore literals for the time being
#[rustfmt::skip]
fn word(input: &str) -> IResult<&str, Word> {
    let (remaining, word) = alt(
        (
            full_word,
            bit_spec_as_word,

        )
    )(input)?;

    //let (remaining, (index, bit_spec)) = tuple((opt(index), bit_spec_delimited))(input)?;
    Ok((remaining, word))
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
    fn test_word() {
        let data = "3[2..6]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: Some(3),
                bit_spec: Some(BitSpec::Range(2, 6))
            }
        );

        let data = "4[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: Some(4),
                bit_spec: None
            }
        );

        let data = "[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: None,
                bit_spec: None
            }
        );

        let data = "3[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: Some(3),
                bit_spec: None
            }
        );

        let data = "7";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: None,
                bit_spec: Some(BitSpec::Single(7))
            }
        )
    }

    #[test]
    fn test_full_word() {
        let data = "3[4..6]";
        let (_, r) = full_word(data).unwrap();

        let expected_word = Word {
            index: Some(3),
            bit_spec: Some(BitSpec::Range(4, 6)),
        };

        assert_eq!(r, expected_word);

        let data = "9[]";
        let (_, r) = full_word(data).unwrap();
        let expected_word = Word {
            index: Some(9),
            bit_spec: None,
        };
        assert_eq!(r, expected_word);

        let data = "[3..7]";
        let (_, r) = full_word(data).unwrap();
        let expected_word = Word {
            index: None,
            bit_spec: Some(BitSpec::Range(3, 7)),
        };
        assert_eq!(r, expected_word);

        let data = "[]";
        let (_, r) = full_word(data).unwrap();
        let expected_word = Word {
            index: None,
            bit_spec: None,
        };
        assert_eq!(r, expected_word);
    }

    // #[test]
    // fn test_bit_spec_delimited() {
    //     let data = "[3..5]";
    //     let (_, r) = bit_spec_delimited(data).unwrap();
    //     assert_eq!(r, BitSpec::Range(3, 5));

    //     let data = "[3]";
    //     let (_, r) = bit_spec_delimited(data).unwrap();
    //     assert_eq!(r, BitSpec::Single(3));

    //     let data = "[ ]";
    //     let (_, r) = bit_spec_delimited(data).unwrap();
    //     assert_eq!(r, BitSpec::WholeWord);

    //     let data = "3..4";
    //     assert!(bit_spec_delimited(data).is_err());
    // }
    // #[test]
    // fn test_empty_bit_spec() {
    //     let data = "[]";

    //     let (_, r) = empty_bit_spec(data).unwrap();
    //     assert_eq!(r, BitSpec::WholeWord);

    //     let data = "[ ]";

    //     let (_, r) = empty_bit_spec(data).unwrap();
    //     assert_eq!(r, BitSpec::WholeWord);

    //     let data = "[    ]";

    //     let (_, r) = empty_bit_spec(data).unwrap();
    //     assert_eq!(r, BitSpec::WholeWord);

    //     let data = "[3]";
    //     assert!(empty_bit_spec(data).is_err());
    // }

    #[test]
    fn test_bit_spec() {
        let data = "4..6";
        let (_, r) = bit_spec(data).unwrap();
        assert_eq!(r, BitSpec::Range(4, 6));

        let data = "7";
        let (_, r) = bit_spec(data).unwrap();
        assert_eq!(r, BitSpec::Single(7));
    }

    #[test]
    fn test_single_bit() {
        let data = "2";
        let (_, r) = single_bit(data).unwrap();
        assert_eq!(r, BitSpec::Single(2));
    }

    #[test]
    fn test_range() {
        let data = "2..45";
        let (_, r) = range(data).unwrap();
        assert_eq!(r, BitSpec::Range(2, 45));
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

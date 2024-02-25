#[allow(unused_imports)]
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take_while1},
    character::complete::{char, one_of, space0, u8},
    character::is_digit,
    combinator::{into, map, recognize, value},
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
    WholeWord,
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Word {
    index: u8,
    bit_spec: BitSpec,
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
    alt((range, single_bit))(input) // Order inportant
}

fn empty_bit_spec(input: &str) -> IResult<&str, BitSpec> {
    // Required delimiters
    let (remaining, _) = delimited(tag("["), space0, tag("]"))(input)?;
    Ok((remaining, BitSpec::WholeWord))
}

fn bit_spec_delimited(input: &str) -> IResult<&str, BitSpec> {
    let (remaining, bit_spec) =
        alt((delimited(tag("["), bit_spec, tag("]")), empty_bit_spec))(input)?;
    Ok((remaining, bit_spec))
}

// word = index bits_spec_delimited | index "[" literal "]";
// TODO Ignore literals for the time being
fn word(input: &str) -> IResult<&str, Word> {
    let (remaining, (index, bit_spec)) = tuple((index, bit_spec_delimited))(input)?;
    Ok((remaining, Word { index, bit_spec }))
}

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
                index: 3,
                bit_spec: BitSpec::Range(2, 6)
            }
        );

        let data = "3[6]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 3,
                bit_spec: BitSpec::Single(6)
            }
        );

        let data = "4[]";
        let (_, r) = word(data).unwrap();
        assert_eq!(
            r,
            Word {
                index: 4,
                bit_spec: BitSpec::WholeWord
            }
        );

        let data = "[]";
        assert!(word(data).is_err());

        let data = "4";
        assert!(word(data).is_err());
    }

    #[test]
    fn test_bit_spec_delimited() {
        let data = "[3..5]";
        let (_, r) = bit_spec_delimited(data).unwrap();
        assert_eq!(r, BitSpec::Range(3, 5));

        let data = "[3]";
        let (_, r) = bit_spec_delimited(data).unwrap();
        assert_eq!(r, BitSpec::Single(3));

        let data = "[ ]";
        let (_, r) = bit_spec_delimited(data).unwrap();
        assert_eq!(r, BitSpec::WholeWord);

        let data = "3..4";
        assert!(bit_spec_delimited(data).is_err());
    }
    #[test]
    fn test_empty_bit_spec() {
        let data = "[]";

        let (_, r) = empty_bit_spec(data).unwrap();
        assert_eq!(r, BitSpec::WholeWord);

        let data = "[ ]";

        let (_, r) = empty_bit_spec(data).unwrap();
        assert_eq!(r, BitSpec::WholeWord);

        let data = "[    ]";

        let (_, r) = empty_bit_spec(data).unwrap();
        assert_eq!(r, BitSpec::WholeWord);

        let data = "[3]";
        assert!(empty_bit_spec(data).is_err());
    }

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

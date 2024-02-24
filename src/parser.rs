use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take_while1},
    character::complete::{char, one_of},
    character::is_digit,
    combinator::{map, recognize},
    multi::{many0, many1},
    sequence::{preceded, terminated, tuple},
    IResult, Parser,
};
enum BitRange {
    Single(usize),
    Range(usize, usize),
}
fn index(input: &str) -> IResult<&str, &str> {
    is_a("0123456789")(input)
}

fn range(input: &str) -> IResult<&str, (&str, &str, &str)> {
    tuple((index, tag(".."), index))(input)
}

// fn bits(input: &str) -> IResult<&str, &str> {
//     alt((index, range))(input)
// }

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
    fn test_range() {
        let data = "2..45";
        let (_, r) = range(data).unwrap();
        assert_eq!(r, ("2", "..", "45"));
    }

    #[test]
    fn test_index() {
        let data = "34";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, "34");

        let data = "7";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, "7");

        let data = "48";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, "48");

        let data = "2345;";
        let (_, i) = index(data).unwrap();
        assert_eq!(i, "2345");
    }

    #[test]
    fn test_hexadecimal() {
        let data = "0x45B7";
        let (_, hex) = hexadecimal(data).unwrap();

        assert_eq!(hex, "45B7");
    }
}

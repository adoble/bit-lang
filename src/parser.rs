use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::recognize,
    multi::{many0, many1},
    sequence::{preceded, terminated},
    IResult, Parser,
};
// use nom::number::complete::be_u16;

pub fn hex_header(input: &str) -> IResult<&str, &str> {
    tag("0x")(input)
}

// pub fn length_value(input: &[u8]) -> IResult<&[u8],&[u8]> {
//     let (input, length) = be_u16(input)?;
//     take(length)(input)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hex_header() {
        let data = "0x4567";

        let (remaining, header) = hex_header(data).unwrap();

        assert_eq!(header, "0x");
    }

    #[test]
    fn test_hexadecimal() {
        let data = "0x45B7";
        let (_, hex) = hexadecimal(data).unwrap();

        assert_eq!(hex, "45B7");
    }
}

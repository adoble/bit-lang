use nom::bytes::complete::{tag, take};
use nom::number::complete::be_u16;
use nom::IResult;

pub fn hex_header(input: &str) -> IResult<&str, &str> {
    tag("0x")(input)
}

// pub fn length_value(input: &[u8]) -> IResult<&[u8],&[u8]> {
//     let (input, length) = be_u16(input)?;
//     take(length)(input)
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hex_header() {
        let data = "0x4567";

        let (remaining, header) = hex_header(data).unwrap();

        assert_eq!(header, "0x");
    }
}

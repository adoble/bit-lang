use std::f32::consts::E;

use crate::parser::*;

mod parser;

fn main() {
    let data = "0x4567";

    let r = hex_header(data);

    match r {
        Ok((remaining, output)) => {
            println!("{:?} {:?}", remaining, output)
        }
        Err(e) => println!("{:?}", e),
    }

    //println!("{:?}", r.0);
}

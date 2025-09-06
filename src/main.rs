mod json_parser;

use json_parser::JsonParser;
use std::io::{Read, stdin};

fn main() {
    let mut input_holder: String = String::new();

    match stdin().read_to_string(&mut input_holder) {
        Ok(_) => {
            let parser: Result<JsonParser, &'static str> = JsonParser::new(input_holder);

            match parser {
                Ok(parser) => {
                    println!("The JSON object entries are:");
                    for (key, val) in parser.get_map().iter() {
                        println!("{}: {}", key, val);
                    }
                }
                Err(err_str) => {
                    println!("{}", err_str);
                }
            }
        }
        Err(err) => {
            eprintln!("{}", err.to_string());
            println!("Invalid input provided!");
        }
    }
}

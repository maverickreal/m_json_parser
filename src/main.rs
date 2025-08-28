mod json_parser;

use json_parser::JsonParser;
use std::io;

fn main() {
    let mut input_holder: String = String::new();

    match io::stdin().read_line(&mut input_holder) {
        Ok(_) => {
            let parser: Result<JsonParser, &'static str> = JsonParser::new(input_holder);

            match parser {
                Ok(parser) => {
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

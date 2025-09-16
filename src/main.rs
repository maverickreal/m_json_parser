mod json_parser;

use json_parser::JsonParser;
use std::{
    collections::HashMap,
    io::{Read, stdin},
};

use crate::json_parser::JsonValueTypes;

fn print_parser_arr(arr: &Vec<JsonValueTypes>, level: usize) -> () {
    let level_indent: String = "\t".repeat(level);
    println!("{}{}", level_indent, "[");

    for val in arr.iter() {
        if let JsonValueTypes::Object(inner_map) = val {
            print_parser_map(inner_map, level);
        } else if let JsonValueTypes::Array(inner_arr) = val {
            print_parser_arr(inner_arr, level);
        } else {
            println!("{}{},", level_indent, val.to_string().unwrap());
        }
    }

    println!("{}{}", level_indent, "]");
}

fn print_parser_map(map: &HashMap<String, JsonValueTypes>, level: usize) -> () {
    if level == 0 {
        println!("The JSON object entries are:");
    }
    let level_indent: String = "\t".repeat(level);
    println!("{}{}", level_indent, "{");

    for (key, val) in map.iter() {
        if let JsonValueTypes::Object(inner_map) = val {
            println!("{}{}:", level_indent, key);
            print_parser_map(inner_map, level + 1);
        } else if let JsonValueTypes::Array(inner_arr) = val {
            print_parser_arr(inner_arr, level + 1);
        } else {
            println!("{}{}: {},", level_indent, key, val.to_string().unwrap());
        }
    }

    if level > 0 {
        println!("{}{},", level_indent, "}");
    } else {
        println!("{}", "}");
    }
}

fn main() {
    let mut input_holder: String = String::new();

    match stdin().read_to_string(&mut input_holder) {
        Ok(_) => {
            let parser: Result<JsonParser, &'static str> = JsonParser::new(input_holder);

            match parser {
                Ok(parser) => {
                    print_parser_map(parser.get_map(), 0);
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

mod json_parser;

use json_parser::JsonParser;
use std::collections::HashMap;

use crate::json_parser::JsonValueTypes;

fn print_parser_arr(arr: &Vec<JsonValueTypes>, level: usize) -> () {
    let level_indent: String = "\t".repeat(level);
    println!("{level_indent}[");

    for val in arr.iter() {
        if let JsonValueTypes::Object(inner_map) = val {
            print_parser_map(inner_map, level + 1);
        } else if let JsonValueTypes::Array(inner_arr) = val {
            print_parser_arr(inner_arr, level + 1);
        } else {
            println!("{level_indent}\t{},", val.to_string().unwrap());
        }
    }

    println!("{level_indent}],");
}

fn print_parser_map(map: &HashMap<String, JsonValueTypes>, level: usize) -> () {
    if level == 0 {
        println!("The JSON object entries are:");
    }
    let level_indent: String = "\t".repeat(level);
    println!("{level_indent}{}", "{");

    for (key, val) in map.iter() {
        if let JsonValueTypes::Object(inner_map) = val {
            println!("{level_indent}\t{key}:");
            print_parser_map(inner_map, level + 2);
        } else if let JsonValueTypes::Array(inner_arr) = val {
            println!("{level_indent}\t{key}:");
            print_parser_arr(inner_arr, level + 2);
        } else {
            println!("{level_indent}\t{key}: {},", val.to_string().unwrap());
        }
    }

    if level > 0 {
        println!("{level_indent}{},", "}");
    } else {
        println!("{}", "}");
    }
}

fn main() {
    let test_str: String = String::from(
        r#"{"a":[1, [+12.3E-10, null]], "b\u1234":{"c"   :{"d":"\te\nh"}, "d":false}   ,"c": [1, {"b":"c"}], "d":{"e":"f", "g":[1, 2]}}"#,
    );
    let parser: Result<JsonParser, &'static str> = JsonParser::new(test_str);

    match parser {
        Ok(parser) => {
            print_parser_map(parser.get_map(), 0);
        }
        Err(err_str) => {
            println!("{}", err_str);
        }
    }
}

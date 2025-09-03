use std::collections::HashMap;

const NOT_VALID_JSON_FORMAT: &str = "The provided data is not in a valid JSON format!";

/*enum ParsePhase {
    KeyString,
    ValueString,
    ValueNumber,
    ValueBoolean,
    ValueNull,
    ValueArray,
    ValueObject,
    ObjectSpace,
}*/

pub struct JsonParser {
    text: String,
    map: HashMap<String, String>,
}

impl JsonParser {
    pub fn new(_text: String) -> Result<Self, &'static str> {
        let chars_array: Vec<char> = _text.trim().chars().collect::<Vec<char>>();
        let mut map: HashMap<String, String> = HashMap::new();
        let ended_at: usize = Self::parse(&chars_array, 0, &mut map)?;

        if ended_at != (chars_array.len() - 1) {
            // e
            println!(
                "{}",
                "Some error occurred while parsing the JSON data!\n
                Possible causes:\n\t- Data present after the end of 
                the JSON object in the file!"
            );
            return Err(NOT_VALID_JSON_FORMAT);
        }

        let instance: JsonParser = JsonParser {
            text: _text,
            map: map,
        };

        return Ok(instance);
    }

    pub fn get_text(&self) -> &String {
        return &self.text;
    }

    pub fn get_map(&self) -> &HashMap<String, String> {
        return &self.map;
    }

    fn parse_special_char(
        parsed_unicode_str: &mut String,
        chars_arr: &Vec<char>,
        mut ind: usize,
    ) -> Result<usize, &'static str> {
        let mut unicode_rem_cnt: i8 = 0;

        while ind < chars_arr.len() {
            let ch: char = chars_arr[ind];

            if unicode_rem_cnt < 0 || unicode_rem_cnt > 4 {
                // e
                println!(
                    "{}",
                    "Some error occurred while parsing the JSON data!\n
                        Possible causes:\n\t- Invalid unicode sequence!"
                );
                return Err(NOT_VALID_JSON_FORMAT);
            }

            if unicode_rem_cnt > 0 {
                parsed_unicode_str.push(ch);
                unicode_rem_cnt -= 1;

                if unicode_rem_cnt != 0 {
                    ind += 1;
                    continue;
                }

                if parsed_unicode_str.len() != 4 {
                    // e
                    println!("{}", "Invalid unicode sequence encountered!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                match u32::from_str_radix(&parsed_unicode_str, 16) {
                    Ok(num) => {
                        if let Some(code) = std::char::from_u32(num) {
                            *parsed_unicode_str = code.to_string();
                            break;
                        } else {
                            // e
                            println!("{}", "Invalid unicode sequence encountered!");
                            return Err(NOT_VALID_JSON_FORMAT);
                        }
                    }
                    Err(err) => {
                        // e
                        println!("{}: {}", "Invalid unicode sequence encountered!", err);
                        return Err(NOT_VALID_JSON_FORMAT);
                    }
                }
            }

            let escaped_char = match ch {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'b' => '\x08',
                'f' => '\x0C',
                'u' => {
                    unicode_rem_cnt = 4;
                    ind += 1;
                    continue;
                }
                _ => {
                    // e
                    println!("{}", "Invalid escape sequence encountered!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }
            };

            *parsed_unicode_str = escaped_char.to_string();
            break;
        }

        return Ok(ind);
    }

    fn extract_string(
        val_str: &mut String,
        chars_arr: &Vec<char>,
        mut ind: usize,
    ) -> Result<usize, &'static str> {
        let ch_arr_sz: usize = chars_arr.len();
        val_str.push('"');

        while ind < ch_arr_sz {
            let ch: char = chars_arr[ind];

            if ch == '"' {
                break;
            }

            if ch != '\\' {
                ind += 1;
                val_str.push(ch);
                continue;
            }

            let mut parsed_unicode_str: String = String::new();
            let spec_char_end: usize =
                Self::parse_special_char(&mut parsed_unicode_str, chars_arr, ind + 1)?;
            val_str.push_str(&parsed_unicode_str);
            ind = spec_char_end + 1;
        }

        val_str.push('"');

        return Ok(ind);
    }

    fn parse(
        chars_array: &Vec<char>,
        mut ind: usize,
        map: &mut HashMap<String, String>,
    ) -> Result<usize, &'static str> {
        let mut stack: Vec<String> = Vec::new();
        /*let mut escape: bool = false;
        let mut unicode_seq: String = String::new();
        let mut cur_phase: ParsePhase = ParsePhase::ObjectSpace;
        let mut unicode_rem_cnt: i8 = 0;

        let in_string_phase = |_cur_phase: &ParsePhase| -> bool {
            return matches!(_cur_phase, ParsePhase::KeyString | ParsePhase::ValueString);
        };*/

        while ind < chars_array.len() {
            let ch: char = chars_array[ind];

            if ch == '{' {
                /*if in_string_phase(&cur_phase) {
                    stack.last_mut().unwrap().push(ch);
                } else {*/
                if !stack.is_empty() {
                    // e
                    println!("{}", "Stack isn't empty!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                stack.push(String::from(ch));
                // }
                continue;
            }

            if stack.is_empty() {
                // e
                println!("{}", "Stack is empty!");
                return Err(NOT_VALID_JSON_FORMAT);
            }

            // let last_element: String = stack.last().unwrap().clone();

            if ch == '"' {
                let mut val_str: String = String::new();
                let str_end: usize = Self::extract_string(&mut val_str, &chars_array, ind + 1)?;
                stack.push(val_str);
                ind = str_end + 1;

                if stack.len() == 4 {
                    map.insert(stack[1].to_string(), stack[3].to_string());
                    stack.drain(1..);
                } else if stack.len() != 2 {
                    // e
                    println!("{}", "Stack length is neither 2 nor 4!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                continue;

                /*if in_string_phase(&cur_phase) {
                    cur_phase = ParsePhase::ObjectSpace;
                    stack.last_mut().unwrap().push(ch);

                    if stack.len() == 4 {
                        map.insert(stack[1].to_string(), stack[3].to_string());
                        stack.drain(1..);
                    } else if stack.len() != 2 {
                        // e
                        println!("{}", "Stack length is neither 2 nor 4!");
                        return Err(NOT_VALID_JSON_FORMAT);
                    }
                } else if last_element == "{" {
                    cur_phase = ParsePhase::KeyString;
                } else if last_element == ":" {
                    cur_phase = ParsePhase::ValueString;
                } else {
                    // e
                    println!("{}: {}", "Invalid character encountered! (1)", last_element);
                    return Err(NOT_VALID_JSON_FORMAT);
                }
                continue;*/
            }

            if ch == ':' {
                if stack.len() != 2 {
                    // e
                    println!("{}", "Stack length is not 2!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                stack.push(String::from(ch));
                continue;
            }

            if ch.is_ascii_digit() || ch == '-' {
                continue;
            }

            if ch == 'n' {
                continue;
            }

            if ch == 't' || ch == 'f' {
                continue;
            }

            if ch == '[' {
                continue;
            }

            if ch == '{' {
                continue;
            }

            if ch == ',' {
                continue;
            }

            if ch == '}' {
                break;
            }

            if !ch.is_whitespace() && ch != ',' {
                println!(
                    "Stack isn't empty! (2); char: {}; stack: {:?}; ind: {}; char_arr: {:?};",
                    ch, stack, ind, chars_array
                );
                return Err(NOT_VALID_JSON_FORMAT);
            }

            ind += 1;

            /*if !matches!(cur_phase, ParsePhase::ObjectSpace) {
                if in_string_phase(&cur_phase) {
                    if ch == '\\' {
                        escape = true;
                    } else {
                        stack.last_mut().unwrap().push(ch);
                    }
                } else {
                    match cur_phase {
                        ParsePhase::ValueArray => {}
                        ParsePhase::ValueBoolean => {}
                        ParsePhase::ValueNull => {}
                        ParsePhase::ValueNumber => {}
                        ParsePhase::ValueObject => {}
                        _ => {}
                    }
                }
            } else*/
            /*if ch == ':' {
                stack.push(String::from(ch));
            } else if ch == '-' || ch.is_ascii_digit() {
                if matches!(cur_phase, ParsePhase::ValueNumber) && ch == '-' {
                    // e
                    println!("{}", "Invalid character encountered! (2)");
                    return Err(NOT_VALID_JSON_FORMAT);
                }
                if !matches!(cur_phase, ParsePhase::ValueNumber) {
                    cur_phase = ParsePhase::ValueNumber;
                    stack.push(String::from(ch));
                } else {
                    stack.last_mut().unwrap().push(ch);
                }
            }*/
        }

        if stack.len() != 1 {
            println!(
                "Stack isn't empty! (2); stack: {:?}; ind: {}; char_arr: {:?};",
                stack, ind, chars_array
            );
            return Err(NOT_VALID_JSON_FORMAT);
        }

        return Ok(ind);
    }
}

use std::collections::HashMap;

enum ParsePhase {
    KeyString,
    ValueString,
    ValueNumber,
    ValueBoolean,
    ValueNull,
    ValueArray,
    ValueObject,
    ObjectSpace,
}

pub struct JsonParser {
    text: String,
    map: HashMap<String, String>,
}

impl JsonParser {
    pub fn new(mut _text: String) -> Result<Self, &'static str> {
        _text = String::from(_text.trim());
        let parsed_result: HashMap<String, String> = Self::parse(&_text)?;
        let instance: JsonParser = JsonParser {
            text: _text,
            map: parsed_result,
        };

        return Ok(instance);
    }

    pub fn get_text(&self) -> &String {
        return &self.text;
    }

    pub fn get_map(&self) -> &HashMap<String, String> {
        return &self.map;
    }

    fn parse(_text: &String) -> Result<HashMap<String, String>, &'static str> {
        let chars_array: Vec<char> = _text.chars().collect::<Vec<char>>();
        const NOT_VALID_JSON_FORMAT: &str = "The provided data is not in a valid JSON format!";
        let mut map: HashMap<String, String> = HashMap::new();
        let mut stack: Vec<String> = Vec::new();
        let mut escape: bool = false;
        let mut unicode_seq: String = String::new();
        let mut cur_phase: ParsePhase = ParsePhase::ObjectSpace;
        let mut unicode_rem_cnt: i8 = 0;

        let in_string_phase = |_cur_phase: &ParsePhase| -> bool {
            return matches!(_cur_phase, ParsePhase::KeyString | ParsePhase::ValueString);
        };

        for ind in 0..chars_array.len() {
            let ch: char = chars_array[ind];

            if escape {
                if unicode_rem_cnt < 0 || unicode_rem_cnt > 4 {
                    // e
                    println!(
                        "{}",
                        "Some error occurred while parsing the JSON data!\nPossible causes:\n\t- Invalid unicode sequence!"
                    );
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                if unicode_rem_cnt > 0 {
                    unicode_seq.push(ch);
                    unicode_rem_cnt -= 1;

                    if unicode_rem_cnt == 0 {
                        if unicode_seq.len() == 4 {
                            match u32::from_str_radix(&unicode_seq, 16) {
                                Ok(num) => {
                                    if let Some(code) = std::char::from_u32(num) {
                                        stack.last_mut().unwrap().push(code);
                                        escape = false;
                                        unicode_seq.clear();
                                    } else {
                                        // e
                                        println!("{}", "Invalid unicode sequence encountered!");
                                        return Err(NOT_VALID_JSON_FORMAT);
                                    }
                                }
                                Err(err) => {
                                    // e
                                    println!(
                                        "{}: {}",
                                        "Invalid unicode sequence encountered!", err
                                    );
                                    return Err(NOT_VALID_JSON_FORMAT);
                                }
                            }
                        } else {
                            // e
                            println!("{}", "Invalid unicode sequence encountered!");
                            return Err(NOT_VALID_JSON_FORMAT);
                        }
                    }
                    continue;
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
                        continue;
                    }
                    _ => {
                        // e
                        println!("{}", "Invalid escape sequence encountered!");
                        return Err(NOT_VALID_JSON_FORMAT);
                    }
                };

                stack.last_mut().unwrap().push(escaped_char);
                escape = false;
                continue;
            }

            if ch == '{' {
                if in_string_phase(&cur_phase) {
                    stack.last_mut().unwrap().push(ch);
                } else {
                    if !stack.is_empty() {
                        // e
                        println!("{}", "Stack isn't empty!");
                        return Err(NOT_VALID_JSON_FORMAT);
                    }

                    stack.push(String::from(ch));
                }
                continue;
            }

            if stack.is_empty() {
                // e
                println!("{}", "Stack is empty!");
                return Err(NOT_VALID_JSON_FORMAT);
            }

            let last_element: String = stack.last().unwrap().clone();

            if ch == '"' {
                if in_string_phase(&cur_phase) {
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
                    stack.push(String::from(ch));
                } else if last_element == ":" {
                    cur_phase = ParsePhase::ValueString;
                    stack.push(String::from(ch));
                } else {
                    // e
                    println!("{}: {}", "Invalid character encountered! (1)", last_element);
                    return Err(NOT_VALID_JSON_FORMAT);
                }
                continue;
            }

            if !matches!(cur_phase, ParsePhase::ObjectSpace) {
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
                        ParsePhase::ValueObject => {},
                        _ => {
                            
                        }
                    }
                }
            } else if ch == ':' {
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
            } else {
                let valid_end_brace: bool =
                    ch == '}' && (ind == chars_array.len() - 1) && stack.len() == 1;
                let ignore_char: bool = ch.is_whitespace() || ch == ',';

                if !ignore_char && !valid_end_brace {
                    // e
                    println!(
                        "Stack isn't empty! (2); char: {}; stack: {:?}; ind: {}; char_arr: {:?};",
                        ch, stack, ind, chars_array
                    );
                    return Err(NOT_VALID_JSON_FORMAT);
                }
            }
        }

        return Ok(map);
    }
}

use std::collections::HashMap;

const NOT_VALID_JSON_FORMAT: &str = "The provided data is not in a valid JSON format!";

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

            if unicode_rem_cnt > 0 {
                parsed_unicode_str.push(ch);
                unicode_rem_cnt -= 1;

                if unicode_rem_cnt > 0 {
                    ind += 1;
                    continue;
                }

                let mut is_valid_unicode_seq: bool = true;

                match u32::from_str_radix(&parsed_unicode_str, 16) {
                    Ok(num) => {
                        if let Some(code) = std::char::from_u32(num) {
                            *parsed_unicode_str = code.to_string();
                            break;
                        } else {
                            is_valid_unicode_seq = false;
                        }
                    }
                    Err(_err) => {
                        is_valid_unicode_seq = false;
                    }
                }

                if !is_valid_unicode_seq {
                    // e
                    println!("{}", "Invalid unicode sequence encountered!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }
            }

            let escaped_char = match ch {
                '"' => "\"",
                '\\' => "\\",
                '/' => "/",
                'n' => "\n",
                'r' => "\r",
                't' => "\t",
                'b' => "\x08",
                'f' => "\x0C",
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

    fn is_null_or_bool_sequence(chars_arr: &Vec<char>, ind: usize) -> Result<String, &'static str> {
        if chars_arr[ind] == 'n' {
            let is_null: bool = (ind + 4) < chars_arr.len()
                && chars_arr[ind] == 'n'
                && chars_arr[ind + 1] == 'u'
                && chars_arr[ind + 2] == 'l'
                && chars_arr[ind + 3] == 'l';

            if is_null {
                return Ok(String::from("null"));
            }
        } else if chars_arr[ind] == 't' {
            let is_true: bool = (ind + 4) < chars_arr.len()
                && chars_arr[ind] == 't'
                && chars_arr[ind + 1] == 'r'
                && chars_arr[ind + 2] == 'u'
                && chars_arr[ind + 3] == 'e';

            if is_true {
                return Ok(String::from("true"));
            }
        } else if chars_arr[ind] == 'f' {
            let is_false: bool = (ind + 5) < chars_arr.len()
                && chars_arr[ind] == 'f'
                && chars_arr[ind + 1] == 'a'
                && chars_arr[ind + 2] == 'l'
                && chars_arr[ind + 3] == 's'
                && chars_arr[ind + 4] == 'e';

            if is_false {
                return Ok(String::from("false"));
            }
        }

        return Err(NOT_VALID_JSON_FORMAT);
    }

    fn is_num_sequence(chars_arr: &Vec<char>, mut ind: usize) -> Result<String, &'static str> {
        let mut val_str: String = String::new();
        let mut have_decimal_point: bool = false;
        let mut have_exp_char: bool = false;

        let is_exp_char = |ch: char| -> bool {
            return ch == 'e' || ch == 'E';
        };

        while ind < chars_arr.len() {
            let ch: char = chars_arr[ind];

            if ch == '+' || ch == '-' {
                if val_str.is_empty() || is_exp_char(val_str.chars().last().unwrap()) {
                    val_str.push(ch);
                    ind += 1;
                    continue;
                }
                // e
                println!("{}", "Invalid character in number!");
                return Err(NOT_VALID_JSON_FORMAT);
            }

            if ch.is_ascii_digit() {
                val_str.push(ch);
                ind += 1;
                continue;
            }

            if ch == '.' {
                if have_decimal_point {
                    // e
                    println!("{}", "Multiple decimal points in a number!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                val_str.push(ch);
                have_decimal_point = true;
                ind += 1;
                continue;
            }

            if is_exp_char(ch) {
                if have_exp_char {
                    // e
                    println!("{}", "Multiple decimal points in a number!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                val_str.push(ch);
                have_exp_char = true;
                ind += 1;
                continue;
            }

            break;
        }

        return Ok(val_str);
    }

    fn parse(
        chars_array: &Vec<char>,
        mut ind: usize,
        map: &mut HashMap<String, String>,
    ) -> Result<usize, &'static str> {
        let mut stack: Vec<String> = Vec::new();

        while ind < chars_array.len() {
            let ch: char = chars_array[ind];

            if stack.is_empty() {
                if ch == '{' {
                    stack.push(String::from(ch));
                    ind += 1;
                    continue;
                }
                // e
                println!("{}", "Stack is empty!");
                return Err(NOT_VALID_JSON_FORMAT);
            }

            if ch == '"' {
                let mut val_str: String = String::new();
                let str_end: usize = Self::extract_string(&mut val_str, &chars_array, ind + 1)?;
                stack.push(val_str);

                if stack.len() == 4 {
                    map.insert(stack[1].clone(), stack[3].clone());
                    stack.drain(1..);
                } else if stack.len() != 2 {
                    // e
                    println!("{}", "Stack length is neither 2 nor 4!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                ind = str_end + 1;
                continue;
            }

            if ch == ':' {
                if stack.len() != 2 {
                    // e
                    println!("{}", "Stack length is not 2!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                stack.push(String::from(ch));
                ind += 1;
                continue;
            }

            if ch == 'n' || ch == 't' || ch == 'f' {
                let val_str: String = Self::is_null_or_bool_sequence(&chars_array, ind)?;
                ind = ind + val_str.len();
                stack.push(val_str);
                continue;
            }

            if ch.is_ascii_digit() || ch == '-' || ch == '+' {
                let val_str: String = Self::is_num_sequence(&chars_array, ind)?;
                ind = ind + val_str.len();
                stack.push(val_str);
                continue;
            }

            if ch == '[' {
                /*let val_str: String = Self::is_array_sequence(&chars_array, ind)?;
                ind = ind + val_str.len();
                stack.push(val_str);*/
                continue;
            }

            if ch == '{' {
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

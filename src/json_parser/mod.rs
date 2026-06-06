use std::collections::HashMap;

const NOT_VALID_JSON_FORMAT: &str = "The provided data is not in a valid JSON format!";

#[derive(Clone, Debug)]
pub enum JsonValueTypes {
    String(String),
    Number(f64),
    Object(HashMap<String, JsonValueTypes>),
    Array(Vec<JsonValueTypes>),
    Boolean(bool),
    Null,
}

impl JsonValueTypes {
    pub fn to_string(&self) -> Option<String> {
        match self {
            JsonValueTypes::Number(val) => Some(val.to_string()),
            JsonValueTypes::Boolean(val) => Some(val.to_string()),
            JsonValueTypes::Null => Some(String::from("null")),
            JsonValueTypes::String(val) => Some(val.clone()),
            _ => None,
        }
    }
}

enum ContainerState {
    Object {
        map: HashMap<String, JsonValueTypes>,
        current_key: Option<String>,
        expecting_colon: bool,
    },
    Array {
        list: Vec<JsonValueTypes>,
    },
}

pub struct JsonParser {
    text: String,
    map: HashMap<String, JsonValueTypes>,
}

impl JsonParser {
    pub fn new(_text: String) -> Result<Self, &'static str> {
        let chars_array: Vec<char> = _text.trim().chars().collect::<Vec<char>>();

        let root_val = Self::parse(&chars_array)?;

        if let JsonValueTypes::Object(map) = root_val {
            let instance = JsonParser { text: _text, map };
            return Ok(instance);
        } else {
            return Err("Root of the JSON must be an object.");
        }
    }

    pub fn get_map(&self) -> &HashMap<String, JsonValueTypes> {
        return &self.map;
    }

    fn extract_special_char_util(
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

                match u32::from_str_radix(parsed_unicode_str, 16) {
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
                Self::extract_special_char_util(&mut parsed_unicode_str, chars_arr, ind + 1)?;
            val_str.push_str(&parsed_unicode_str);
            ind = spec_char_end + 1;
        }

        val_str.push('"');

        return Ok(ind);
    }

    fn extract_null_or_bool(chars_arr: &Vec<char>, ind: usize) -> Result<String, &'static str> {
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

    fn extract_number(chars_arr: &Vec<char>, mut ind: usize) -> Result<String, &'static str> {
        let mut val_str: String = String::new();
        let mut have_decimal_point: bool = false;
        let mut have_exp_char: bool = false;

        while ind < chars_arr.len() {
            let ch: char = chars_arr[ind];

            if ch == '+' || ch == '-' {
                let end_val_str_ch: char = val_str.chars().last().unwrap_or_default();

                if val_str.is_empty() || end_val_str_ch == 'e' || end_val_str_ch == 'E' {
                    val_str.push(ch);
                    ind += 1;
                    continue;
                }

                return Err(NOT_VALID_JSON_FORMAT);
            }

            if ch.is_ascii_digit() {
                val_str.push(ch);
                ind += 1;
                continue;
            }

            if ch == '.' {
                if have_decimal_point {
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                val_str.push(ch);
                have_decimal_point = true;
                ind += 1;
                continue;
            }

            if ch == 'e' || ch == 'E' {
                if have_exp_char {
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

    fn add_value_to_stack(
        stack: &mut Vec<ContainerState>,
        val: JsonValueTypes,
    ) -> Result<Option<JsonValueTypes>, &'static str> {
        if let Some(top) = stack.last_mut() {
            match top {
                ContainerState::Object {
                    map,
                    current_key,
                    expecting_colon,
                } => {
                    if let Some(key) = current_key.take() {
                        if !*expecting_colon {
                            return Err(NOT_VALID_JSON_FORMAT);
                        }
                        map.insert(key, val);
                        *expecting_colon = false;
                    } else {
                        if let JsonValueTypes::String(s) = val {
                            *current_key = Some(s);
                        } else {
                            return Err(NOT_VALID_JSON_FORMAT);
                        }
                    }
                }
                ContainerState::Array { list } => {
                    list.push(val);
                }
            }
            Ok(None)
        } else {
            Ok(Some(val))
        }
    }

    fn parse(chars_array: &Vec<char>) -> Result<JsonValueTypes, &'static str> {
        let mut stack: Vec<ContainerState> = Vec::new();
        let mut ind = 0;
        let mut root_value: Option<JsonValueTypes> = None;

        if chars_array.is_empty() {
            return Err(NOT_VALID_JSON_FORMAT);
        }

        while ind < chars_array.len() {
            let ch = chars_array[ind];

            if ch.is_whitespace() || ch == ',' {
                ind += 1;
                continue;
            }

            if ch == ':' {
                if let Some(ContainerState::Object {
                    expecting_colon,
                    current_key,
                    ..
                }) = stack.last_mut()
                {
                    if current_key.is_none() || *expecting_colon {
                        return Err(NOT_VALID_JSON_FORMAT);
                    }
                    *expecting_colon = true;
                } else {
                    return Err(NOT_VALID_JSON_FORMAT);
                }
                ind += 1;

                continue;
            }

            if ch == '{' {
                stack.push(ContainerState::Object {
                    map: HashMap::new(),
                    current_key: None,
                    expecting_colon: false,
                });
                ind += 1;

                continue;
            }

            if ch == '[' {
                stack.push(ContainerState::Array { list: Vec::new() });
                ind += 1;

                continue;
            }

            if ch == '}' {
                if let Some(ContainerState::Object {
                    map, current_key, ..
                }) = stack.pop()
                {
                    if current_key.is_some() {
                        return Err(NOT_VALID_JSON_FORMAT);
                    }
                    let val = JsonValueTypes::Object(map);
                    if let Some(root) = Self::add_value_to_stack(&mut stack, val)? {
                        root_value = Some(root);
                        ind += 1;

                        break;
                    }
                } else {
                    return Err(NOT_VALID_JSON_FORMAT);
                }
                ind += 1;

                continue;
            }

            if ch == ']' {
                if let Some(ContainerState::Array { list }) = stack.pop() {
                    let val = JsonValueTypes::Array(list);

                    if let Some(root) = Self::add_value_to_stack(&mut stack, val)? {
                        root_value = Some(root);
                        ind += 1;

                        break;
                    }
                } else {
                    return Err(NOT_VALID_JSON_FORMAT);
                }
                ind += 1;

                continue;
            }

            if ch == '"' {
                let mut val_str = String::new();
                let end_ind = Self::extract_string(&mut val_str, chars_array, ind + 1)?;
                ind = end_ind + 1;

                if let Some(root) =
                    Self::add_value_to_stack(&mut stack, JsonValueTypes::String(val_str))?
                {
                    root_value = Some(root);
                    break;
                }
                continue;
            }

            if ch == 'n' || ch == 't' || ch == 'f' {
                let val_str = Self::extract_null_or_bool(chars_array, ind)?;
                ind += val_str.len();
                let val = match ch {
                    'n' => JsonValueTypes::Null,
                    _ => JsonValueTypes::Boolean(ch == 't'),
                };

                if let Some(root) = Self::add_value_to_stack(&mut stack, val)? {
                    root_value = Some(root);
                    break;
                }
                continue;
            }

            if ch.is_ascii_digit() || ch == '-' || ch == '+' {
                let val_str = Self::extract_number(chars_array, ind)?;
                ind += val_str.len();
                let val_num: f64 = val_str.parse::<f64>().unwrap();

                if let Some(root) =
                    Self::add_value_to_stack(&mut stack, JsonValueTypes::Number(val_num))?
                {
                    root_value = Some(root);
                    break;
                }
                continue;
            }

            return Err(NOT_VALID_JSON_FORMAT);
        }

        if let Some(root) = root_value {
            while ind < chars_array.len() {
                if !chars_array[ind].is_whitespace() {
                    return Err("Some error occurred while parsing the JSON data!\n\
                    Possible causes:\n\t- Data present after the end of \
                    the JSON object in the file.");
                }
                ind += 1;
            }

            return Ok(root);
        }

        Err(NOT_VALID_JSON_FORMAT)
    }
}

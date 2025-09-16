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

enum IterationScope {
    Object,
    Array,
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

pub struct JsonParser {
    text: String,
    map: HashMap<String, JsonValueTypes>,
}

impl JsonParser {
    pub fn new(_text: String) -> Result<Self, &'static str> {
        let chars_array: Vec<char> = _text.trim().chars().collect::<Vec<char>>();
        let mut map: HashMap<String, JsonValueTypes> = HashMap::new();
        let ended_at: usize = Self::parse_object(&chars_array, 0, &mut map)?;

        if ended_at != (chars_array.len() - 1) {
            return Err("Some error occurred while parsing the JSON data!\n\
                Possible causes:\n\t- Data present after the end of \
                the JSON object in the file.");
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

    fn handle_null_bool_encounter(
        chars_array: &Vec<char>,
        ind: &mut usize,
        container: &mut Vec<JsonValueTypes>,
        ch: char,
        scope: IterationScope,
        map_opt: Option<&mut HashMap<String, JsonValueTypes>>,
    ) -> Result<(), &'static str> {
        let val_str: String = Self::extract_null_or_bool(&chars_array, *ind)?;
        *ind += val_str.len();

        let val = match ch {
            'n' => JsonValueTypes::Null,
            _ => JsonValueTypes::Boolean(ch == 't'),
        };

        container.push(val);

        if matches!(scope, IterationScope::Object) {
            if container.len() == 4 {
                map_opt
                    .unwrap()
                    .insert(container[1].to_string().unwrap(), container[3].clone());
                container.drain(1..);
            } else if container.len() != 2 || !matches!(container[1], JsonValueTypes::String(_)) {
                return Err(NOT_VALID_JSON_FORMAT);
            }
        }

        return Ok(());
    }

    fn handle_number_encounter(
        chars_array: &Vec<char>,
        ind: &mut usize,
        container: &mut Vec<JsonValueTypes>,
        scope: IterationScope,
        map_opt: Option<&mut HashMap<String, JsonValueTypes>>,
    ) -> Result<(), &'static str> {
        let val_str: String = Self::extract_number(&chars_array, *ind)?;
        *ind += val_str.len();
        let val_num: f64 = val_str.parse::<f64>().unwrap();
        container.push(JsonValueTypes::Number(val_num));

        if matches!(scope, IterationScope::Object) {
            if container.len() == 4 {
                map_opt
                    .unwrap()
                    .insert(container[1].to_string().unwrap(), container[3].clone());
                container.drain(1..);
            } else if container.len() != 2 || !matches!(container[1], JsonValueTypes::String(_)) {
                return Err(NOT_VALID_JSON_FORMAT);
            }
        }

        return Ok(());
    }

    fn handle_string_encounter(
        chars_arr: &Vec<char>,
        ind: &mut usize,
        container: &mut Vec<JsonValueTypes>,
        scope: IterationScope,
        map_opt: Option<&mut HashMap<String, JsonValueTypes>>,
    ) -> Result<(), &'static str> {
        let mut val_str: String = String::new();
        let str_end: usize = Self::extract_string(&mut val_str, chars_arr, *ind + 1)?;
        container.push(JsonValueTypes::String(val_str));
        *ind = str_end + 1;

        if matches!(scope, IterationScope::Object) {
            if container.len() == 4 {
                map_opt
                    .unwrap()
                    .insert(container[1].to_string().unwrap(), container[3].clone());
                container.drain(1..);
            } else if container.len() != 2 || !matches!(container[1], JsonValueTypes::String(_)) {
                return Err(NOT_VALID_JSON_FORMAT);
            }
        }

        return Ok(());
    }

    fn handle_array_encounter(
        chars_arr: &Vec<char>,
        ind: &mut usize,
        container: &mut Vec<JsonValueTypes>,
        scope: IterationScope,
        map_opt: Option<&mut HashMap<String, JsonValueTypes>>,
    ) -> Result<(), &'static str> {
        let mut arr_inner: Vec<JsonValueTypes> = Vec::new();
        let end_ind: usize = Self::parse_array(chars_arr, *ind, &mut arr_inner)?;
        container.push(JsonValueTypes::Array(arr_inner));
        *ind = end_ind + 1;

        if matches!(scope, IterationScope::Object) {
            if container.len() == 4 {
                map_opt
                    .unwrap()
                    .insert(container[1].to_string().unwrap(), container[3].clone());
                container.drain(1..);
            } else if container.len() != 2 || !matches!(container[1], JsonValueTypes::String(_)) {
                return Err(NOT_VALID_JSON_FORMAT);
            }
        }

        return Ok(());
    }

    fn handle_object_encounter(
        chars_arr: &Vec<char>,
        ind: &mut usize,
        container: &mut Vec<JsonValueTypes>,
        scope: IterationScope,
        map_opt: Option<&mut HashMap<String, JsonValueTypes>>,
    ) -> Result<(), &'static str> {
        let mut nested_map: HashMap<String, JsonValueTypes> = HashMap::new();
        let end_ind: usize = Self::parse_object(chars_arr, *ind, &mut nested_map)?;
        container.push(JsonValueTypes::Object(nested_map));
        *ind = end_ind + 1;

        if matches!(scope, IterationScope::Object) {
            if container.len() == 4 {
                map_opt
                    .unwrap()
                    .insert(container[1].to_string().unwrap(), container[3].clone());
                container.drain(1..);
            } else if container.len() != 2 || !matches!(container[1], JsonValueTypes::String(_)) {
                return Err(NOT_VALID_JSON_FORMAT);
            }
        }

        return Ok(());
    }

    fn parse_object(
        chars_array: &Vec<char>,
        mut ind: usize,
        map: &mut HashMap<String, JsonValueTypes>,
    ) -> Result<usize, &'static str> {
        let mut stack: Vec<JsonValueTypes> = Vec::new();

        while ind < chars_array.len() {
            let ch: char = chars_array[ind];

            if stack.is_empty() {
                if ch == '{' {
                    stack.push(JsonValueTypes::String(ch.to_string()));
                    ind += 1;
                    continue;
                }

                return Err(NOT_VALID_JSON_FORMAT);
            }

            if ch == '"' {
                Self::handle_string_encounter(
                    &chars_array,
                    &mut ind,
                    &mut stack,
                    IterationScope::Object,
                    Some(map),
                )?;
                continue;
            }

            if ch == ':' {
                if stack.len() != 2 {
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                stack.push(JsonValueTypes::String(String::from(ch)));
                ind += 1;
                continue;
            }

            if ch == 'n' || ch == 't' || ch == 'f' {
                Self::handle_null_bool_encounter(
                    &chars_array,
                    &mut ind,
                    &mut stack,
                    ch,
                    IterationScope::Object,
                    Some(map),
                )?;
                continue;
            }

            if ch.is_ascii_digit() || ch == '-' || ch == '+' {
                Self::handle_number_encounter(
                    &chars_array,
                    &mut ind,
                    &mut stack,
                    IterationScope::Object,
                    Some(map),
                )?;
                continue;
            }

            if ch == '[' {
                Self::handle_array_encounter(
                    &chars_array,
                    &mut ind,
                    &mut stack,
                    IterationScope::Object,
                    Some(map),
                )?;
                continue;
            }

            if ch == '{' {
                Self::handle_object_encounter(
                    &chars_array,
                    &mut ind,
                    &mut stack,
                    IterationScope::Object,
                    Some(map),
                )?;
                continue;
            }

            if ch == '}' {
                break;
            }

            if !ch.is_whitespace() && ch != ',' {
                return Err(NOT_VALID_JSON_FORMAT);
            }

            ind += 1;
        }

        if stack.len() != 1 {
            return Err(NOT_VALID_JSON_FORMAT);
        }

        return Ok(ind);
    }

    fn parse_array(
        chars_array: &Vec<char>,
        mut ind: usize,
        arr: &mut Vec<JsonValueTypes>,
    ) -> Result<usize, &'static str> {
        ind += 1;

        while ind < chars_array.len() {
            let ch: char = chars_array[ind];

            if ch == '"' {
                Self::handle_string_encounter(
                    &chars_array,
                    &mut ind,
                    arr,
                    IterationScope::Array,
                    None,
                )?;
                continue;
            }

            if ch == 'n' || ch == 't' || ch == 'f' {
                Self::handle_null_bool_encounter(
                    &chars_array,
                    &mut ind,
                    arr,
                    ch,
                    IterationScope::Array,
                    None,
                )?;
                continue;
            }

            if ch.is_ascii_digit() || ch == '-' || ch == '+' {
                Self::handle_number_encounter(
                    &chars_array,
                    &mut ind,
                    arr,
                    IterationScope::Array,
                    None,
                )?;
                continue;
            }

            if ch == '[' {
                Self::handle_array_encounter(
                    &chars_array,
                    &mut ind,
                    arr,
                    IterationScope::Array,
                    None,
                )?;
                continue;
            }

            if ch == '{' {
                Self::handle_object_encounter(
                    &chars_array,
                    &mut ind,
                    arr,
                    IterationScope::Array,
                    None,
                )?;
                continue;
            }

            if ch == ']' {
                break;
            }

            if !ch.is_whitespace() && ch != ',' {
                return Err(NOT_VALID_JSON_FORMAT);
            }

            ind += 1;
        }

        if arr.len() != 1 {
            return Err(NOT_VALID_JSON_FORMAT);
        }

        return Ok(ind);
    }
}

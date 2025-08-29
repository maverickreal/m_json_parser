use std::collections::HashMap;

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

        for ind in 0..chars_array.len() {
            let ch: char = chars_array[ind];

            if ch == '{' {
                if !stack.is_empty() {
                    // e
                    println!("{}", "Stack isn't empty!");
                    return Err(NOT_VALID_JSON_FORMAT);
                }

                stack.push(String::from("{"));
                continue;
            }

            if stack.is_empty() {
                // e
                println!("{}", "Stack is empty!");
                return Err(NOT_VALID_JSON_FORMAT);
            }

            let last_element: String = stack.last().unwrap().clone();

            let last_ele_ends_text: bool = last_element
                .chars()
                .last()
                .map_or(false,
                    |c| c.is_alphanumeric() || c.is_whitespace());

            if ch == '"' {
                if last_ele_ends_text {
                    stack.last_mut().unwrap().push(ch);

                    if stack.len() == 4 {
                        map.insert(stack[1].to_string(), stack[3].to_string());
                        stack.drain(1..);
                    } else if stack.len() != 2 {
                        // e
                        println!("{}", "Stack length is neither 2 nor 4!");
                        return Err(NOT_VALID_JSON_FORMAT);
                    }
                } else if last_element == "{" || last_element == ":" {
                    stack.push(String::from(ch));
                } else {
                    // e
                    println!("{}: {}", "Invalid character encountered! (1)", last_element);
                    return Err(NOT_VALID_JSON_FORMAT);
                }
                continue;
            }

            if ch.is_alphanumeric() {
                stack.last_mut().unwrap().push(ch);
                continue;
            }

            if ch.is_whitespace() {
                if last_element == "\"" || last_ele_ends_text {
                    stack.last_mut().unwrap().push(ch);
                }
                continue;
            }

            if ch == ':' {
                stack.push(String::from(ch));
                continue;
            }
            
            if ch == ',' {
                continue;
            }

            if ch != '}' || (ind != chars_array.len() - 1) || stack.len() != 1 {
                // e
                println!(
                    "Stack isn't empty! (2); char: {}; stack: {:?}; ind: {}; char_arr: {:?};",
                    ch, stack, ind, chars_array
                );
                return Err(NOT_VALID_JSON_FORMAT);
            }
        }

        return Ok(map);
    }
}

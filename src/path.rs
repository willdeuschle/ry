use std::fmt;

#[derive(PartialEq)]
enum PathElem {
    Char,
    Dot,
    Quote,
    ArrayOpen,
    ArrayClose,
    EOW,
}

fn char_is(c: char) -> PathElem {
    match c {
        '.' => PathElem::Dot,
        '"' => PathElem::Quote,
        '[' => PathElem::ArrayOpen,
        ']' => PathElem::ArrayClose,
        _ => PathElem::Char,
    }
}

fn next_special_char_is(s: &str) -> (PathElem, usize) {
    for (idx, c) in s.chars().enumerate() {
        let path_elem = char_is(c);
        if path_elem != PathElem::Char {
            return (path_elem, idx);
        }
    }
    (PathElem::EOW, s.len())
}

fn next_specific_special_char(s: &str, pe: PathElem) -> (bool, usize) {
    for (idx, c) in s.chars().enumerate() {
        if char_is(c) == pe {
            return (true, idx);
        }
    }
    return (false, 0);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError(String);

impl ParseError {
    pub fn new(s: &str) -> ParseError {
        ParseError(s.to_string())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn parse_path(path: &str) -> Result<Vec<String>, ParseError> {
    // TODO: finish testing all cases, finish remaining edge cases and error handling
    // can handle errors by returning an option or error type
    let mut parsed_path: Vec<String> = vec![];
    let mut current_idx = 0;
    while current_idx < path.len() {
        match next_special_char_is(&path[current_idx..]) {
            (PathElem::Dot, relative_dot_idx) => {
                let dot_idx = current_idx + relative_dot_idx;
                if dot_idx == current_idx {
                    current_idx += 1;
                    continue;
                }
                parsed_path.push(path[current_idx..dot_idx].to_string());
                current_idx = dot_idx + 1;
            }
            (PathElem::Quote, relative_start_quote_idx) => {
                let start_quoted_word_idx = current_idx + 1 + relative_start_quote_idx;
                let (found, relative_end_quote_idx) =
                    next_specific_special_char(&path[start_quoted_word_idx..], PathElem::Quote);
                if found {
                    let end_quote_idx = start_quoted_word_idx + relative_end_quote_idx;
                    parsed_path.push(path[start_quoted_word_idx..end_quote_idx].to_string());
                    current_idx = end_quote_idx + 1;
                } else {
                    return Err(ParseError::new("invalid path, no closing quote"));
                }
            }
            (PathElem::ArrayOpen, relative_array_open_idx) => {
                let array_open_idx = current_idx + relative_array_open_idx;
                if array_open_idx != current_idx {
                    parsed_path.push(path[current_idx..array_open_idx].to_string());
                }
                let (found, relateive_array_close_idx) =
                    next_specific_special_char(&path[array_open_idx..], PathElem::ArrayClose);
                if found {
                    let array_close_idx = array_open_idx + relateive_array_close_idx;
                    parsed_path.push(path[array_open_idx..array_close_idx + 1].to_string());
                    current_idx = array_close_idx + 1;
                } else {
                    return Err(ParseError::new("invalid path, no closing array character"));
                }
            }
            (PathElem::ArrayClose, _) => {
                return Err(ParseError::new(
                    "invalid path, closing array character before opening",
                ));
            }
            (PathElem::Char, c) => {
                return Err(ParseError::new(&format!("invalid path, found char {}", c)));
            }
            (PathElem::EOW, _) => {
                parsed_path.push(path[current_idx..].to_string());
                break;
            }
        }
    }
    Ok(parsed_path)
}

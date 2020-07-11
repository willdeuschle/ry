use crate::traverse::{traverse, VisitedNode};
use log::{debug, error};
use std::fmt;
use yaml_rust::Yaml;

pub const SPLAT: &'static str = "**";

#[derive(Debug, PartialEq)]
pub enum ArrayIndices {
    Star,
    Indices(Vec<usize>),
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

#[derive(PartialEq, Debug)]
enum PathElem {
    Char,
    Dot,
    Quote,
    ArrayOpen,
    ArrayClose,
    ParenOpen,
    ParenClose,
    EOW,
}

fn char_is(c: char) -> PathElem {
    match c {
        '.' => PathElem::Dot,
        '"' => PathElem::Quote,
        '[' => PathElem::ArrayOpen,
        ']' => PathElem::ArrayClose,
        '(' => PathElem::ParenOpen,
        ')' => PathElem::ParenClose,
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
    match pe {
        PathElem::EOW => (true, s.len()),
        _ => (false, 0),
    }
}

// TODO: audit for integration tests
pub fn parse_path_into(path: &str) -> Vec<String> {
    let parsed_path_res = parse_path(path);
    let parsed_path_vec: Vec<String> = match parsed_path_res {
        Ok(_) => parsed_path_res.unwrap(),
        Err(e) => {
            error!("failed to parse path, error: {}", e);
            std::process::exit(1);
        }
    };
    parsed_path_vec
}

// TODO: audit for integration tests
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
                let (found, relative_array_close_idx) =
                    next_specific_special_char(&path[array_open_idx..], PathElem::ArrayClose);
                if found {
                    let array_close_idx = array_open_idx + relative_array_close_idx;
                    parsed_path.push(path[array_open_idx..array_close_idx + 1].to_string());
                    current_idx = array_close_idx + 1;
                } else {
                    return Err(ParseError::new("invalid path, no closing array character"));
                }
            }
            (PathElem::ParenOpen, relative_paren_open_idx) => {
                let paren_open_idx = current_idx + relative_paren_open_idx;
                if paren_open_idx != current_idx {
                    parsed_path.push(path[current_idx..paren_open_idx].to_string());
                }
                let (found, relative_paren_close_idx) =
                    next_specific_special_char(&path[paren_open_idx..], PathElem::ParenClose);
                if found {
                    let paren_close_idx = paren_open_idx + relative_paren_close_idx;
                    parsed_path.push(path[paren_open_idx..paren_close_idx + 1].to_string());
                    current_idx = paren_close_idx + 1;
                } else {
                    return Err(ParseError::new("invalid path, no closing paren character"));
                }
            }
            (PathElem::ArrayClose, _) => {
                return Err(ParseError::new(
                    "invalid path, closing array character before opening",
                ));
            }
            (PathElem::ParenClose, _) => {
                return Err(ParseError::new(
                    "invalid path, closing paren character before opening",
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

pub fn split_child_filter(filter: &str) -> Result<[&str; 2], ParseError> {
    let split_filter: Vec<&str> = filter.split("==").collect();
    if split_filter.len() != 2 {
        return Err(ParseError::new(&format!(
            "invalid child filter: `{}`",
            filter
        )));
    }
    let mut split_filter_array = ["", ""];
    split_filter_array.copy_from_slice(&split_filter);
    Ok(split_filter_array)
}

pub fn parse_array_child_filter(
    path_elem: &str,
    array_node: &Vec<Yaml>,
    is_final_path_elem: bool,
) -> Result<ArrayIndices, ParseError> {
    if path_elem == "*" {
        return Ok(ArrayIndices::Star);
    }
    let filter_key_and_value = crate::path::split_child_filter(path_elem)?;
    let (filter_path, filter_value) = (filter_key_and_value[0], filter_key_and_value[1]);

    // parse filter_path
    let parsed_path = crate::path::parse_path_into(filter_path);
    debug!("parsed path for child filtering: {:?}", parsed_path);

    let mut indices: Vec<usize> = vec![];

    if is_final_path_elem {
        // child value filter
        debug!("running a child value filter");
        for (idx, array_elem) in array_node.iter().enumerate() {
            if matches_pattern(
                &crate::convert::convert_single_node(array_elem),
                filter_value,
            ) {
                debug!("array_elem matched child value filter: {:?}", array_elem);
                indices.push(idx);
            }
        }
        debug!("child value filtering matched indices: {:?}", indices);
    } else {
        // run a traverse search again against each node to determine if this is a valid child path
        debug!("running a child node filter");
        for (idx, array_elem) in array_node.iter().enumerate() {
            let mut visited = Vec::<VisitedNode>::new();
            traverse(
                array_elem,
                "",
                &parsed_path,
                String::new(),
                false,
                &mut visited,
            );
            if visited.len() != 1 {
                debug!(
                    "array_elem did not match child node filter, continuing: {:?}",
                    array_elem
                );
                continue;
            }
            let ref visited_elem = visited[0];
            if matches_pattern(
                &crate::convert::convert_single_node(visited_elem.yml),
                filter_value, // path element for child filter
            ) {
                debug!("array_elem matched child node filter: {:?}", array_elem);
                indices.push(idx);
            }
        }
        debug!("child node filtering matched indices: {:?}", indices);
    }
    Ok(ArrayIndices::Indices(indices))
}

pub fn parse_array_indexing_operation(path_elem: &str) -> Result<ArrayIndices, ParseError> {
    if path_elem == "*" {
        return Ok(ArrayIndices::Star);
    }
    match path_elem.parse::<usize>() {
        Ok(i) => Ok(ArrayIndices::Indices(vec![i])),
        Err(e) => Err(ParseError(format!(
            "unable to parse array index `{:?}`, error: {:?}",
            path_elem, e
        ))),
    }
}

pub fn matches_pattern(v: &str, pattern: &str) -> bool {
    if v == pattern || pattern == SPLAT {
        return true;
    }
    if pattern.ends_with('*') {
        let truncated_p = pattern.trim_end_matches('*');
        if v.starts_with(truncated_p) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_is() {
        assert_eq!(PathElem::Dot, char_is('.'));
        assert_eq!(PathElem::Quote, char_is('"'));
        assert_eq!(PathElem::ArrayOpen, char_is('['));
        assert_eq!(PathElem::ArrayClose, char_is(']'));
        assert_eq!(PathElem::ParenOpen, char_is('('));
        assert_eq!(PathElem::ParenClose, char_is(')'));
        assert_eq!(PathElem::Char, char_is('a'));
    }

    #[test]
    fn test_next_special_char_is() {
        assert_eq!((PathElem::Dot, 4), next_special_char_is("asdf.asdf"));
        assert_eq!((PathElem::Quote, 4), next_special_char_is("asdf\"asdf"));
        assert_eq!((PathElem::ArrayOpen, 4), next_special_char_is("asdf[asdf"));
        assert_eq!((PathElem::ArrayClose, 4), next_special_char_is("asdf]asdf"));
        assert_eq!((PathElem::ParenOpen, 4), next_special_char_is("asdf(asdf"));
        assert_eq!((PathElem::ParenClose, 4), next_special_char_is("asdf)asdf"));
        assert_eq!((PathElem::EOW, 8), next_special_char_is("asdfasdf"));
    }

    #[test]
    fn test_next_specific_special_char_is() {
        assert_eq!(
            (true, 4),
            next_specific_special_char("asdf.asdf", PathElem::Dot)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdf.asdf", PathElem::Quote)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdfasdf", PathElem::Dot)
        );
        assert_eq!(
            (true, 4),
            next_specific_special_char("asdf\"asdf", PathElem::Quote)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdf\"asdf", PathElem::Dot)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdfasdf", PathElem::Quote)
        );
        assert_eq!(
            (true, 4),
            next_specific_special_char("asdf[asdf", PathElem::ArrayOpen)
        );
        assert_eq!(
            (true, 4),
            next_specific_special_char("asdf(asdf", PathElem::ParenOpen)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdf[asdf", PathElem::Dot)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdfasdf", PathElem::ArrayOpen)
        );
        assert_eq!(
            (true, 4),
            next_specific_special_char("asdf]asdf", PathElem::ArrayClose)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdfasdf", PathElem::ParenOpen)
        );
        assert_eq!(
            (true, 4),
            next_specific_special_char("asdf)asdf", PathElem::ParenClose)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdf]asdf", PathElem::Dot)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdfasdf", PathElem::ArrayClose)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdfasdf", PathElem::ParenClose)
        );
        assert_eq!(
            (true, 0),
            next_specific_special_char("asdfasdf", PathElem::Char)
        );
        assert_eq!(
            (false, 0),
            next_specific_special_char("asdfasdf", PathElem::Dot)
        );
        assert_eq!(
            (true, 8),
            next_specific_special_char("asdfasdf", PathElem::EOW)
        );
    }

    #[test]
    fn test_matches_pattern_identical() {
        assert!(matches_pattern("rusty", "rusty"));
    }

    #[test]
    fn test_matches_pattern_splat() {
        assert!(matches_pattern("rusty", "r*"));
    }

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(matches_pattern("rusty", "**"));
    }

    #[test]
    fn test_matches_pattern_no() {
        assert!(!matches_pattern("rusty", "smooth"));
    }

    #[test]
    fn test_split_child_filter_valid() {
        let split_filter = split_child_filter(".==crabby").unwrap();
        assert_eq!(split_filter[0], ".");
        assert_eq!(split_filter[1], "crabby");
    }

    #[test]
    fn test_split_child_filter_invalid() {
        let split_filter = split_child_filter(".=crabby");
        assert_eq!(true, split_filter.is_err());
    }

    #[test]
    fn test_parse_array_child_filter_star() {
        assert_eq!(
            ArrayIndices::Star,
            parse_array_child_filter("*", &vec![Yaml::Null], false).unwrap()
        );
    }

    #[test]
    fn test_parse_array_child_filter_final() {
        assert_eq!(
            ArrayIndices::Indices(vec![0, 2]),
            parse_array_child_filter(
                ".==dog*",
                &vec![
                    Yaml::String("dog".to_string()),
                    Yaml::String("cat".to_string()),
                    Yaml::String("doggerino".to_string())
                ],
                true
            )
            .unwrap()
        );
    }

    #[test]
    fn test_parse_array_child_filter_node() {
        use yaml_rust::YamlLoader;
        let docs_str = "
- b:
    a1: 1
    d: dog
- b:
    a2: 2
    d: cat
- b:
    a3: 3
    d: doggerino";
        let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

        let array = match doc {
            Yaml::Array(v) => v,
            _ => panic!("invalid doc, not an array"),
        };
        assert_eq!(
            ArrayIndices::Indices(vec![0, 2]),
            parse_array_child_filter("b.d==dog*", array, false).unwrap()
        );
    }

    #[test]
    fn test_parse_array_child_filter_invalid() {
        use yaml_rust::YamlLoader;
        let docs_str = "
- b";
        let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

        let array = match doc {
            Yaml::Array(v) => v,
            _ => panic!("invalid doc, not an array"),
        };
        assert_eq!(true, parse_array_child_filter(".=b", array, false).is_err());
    }

    #[test]
    fn test_parse_array_indexing_operation_wildcard() {
        assert_eq!(
            ArrayIndices::Star,
            parse_array_indexing_operation("*").unwrap()
        );
    }

    #[test]
    fn test_parse_array_indexing_operation_number_path_elem() {
        assert_eq!(
            ArrayIndices::Indices(vec![4]),
            parse_array_indexing_operation("4").unwrap()
        );
    }

    #[test]
    fn test_parse_array_indexing_operation_fails_invalid() {
        assert_eq!(true, parse_array_indexing_operation("a").is_err());
    }
}

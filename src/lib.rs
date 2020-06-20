use std::fmt;
use yaml_rust::{Yaml, YamlEmitter};
use yaml_rust::emitter::{EmitResult, EmitError};
use log::{debug, error};

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
            return (path_elem, idx)
        }
    }
    (PathElem::EOW, s.len())
}

fn next_specific_special_char(s: &str, pe: PathElem) -> (bool, usize) {
    for (idx, c) in s.chars().enumerate() {
        if char_is(c) == pe {
            return (true, idx)
        }
    }
    return (false, 0)
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
            },
            (PathElem::Quote, relative_start_quote_idx) => {
                let start_quoted_word_idx = current_idx + 1 + relative_start_quote_idx;
                let (found, relative_end_quote_idx) = next_specific_special_char(&path[start_quoted_word_idx..], PathElem::Quote);
                if found {
                    let end_quote_idx = start_quoted_word_idx + relative_end_quote_idx;
                    parsed_path.push(path[start_quoted_word_idx..end_quote_idx].to_string());
                    current_idx = end_quote_idx + 1;
                } else {
                    return Err(ParseError::new("invalid path, no closing quote"));
                }
            },
            (PathElem::ArrayOpen, relative_array_open_idx) => {
                let array_open_idx = current_idx + relative_array_open_idx;
                if array_open_idx != current_idx {
                    parsed_path.push(path[current_idx..array_open_idx].to_string());
                }
                let (found, relateive_array_close_idx) = next_specific_special_char(&path[array_open_idx..], PathElem::ArrayClose);
                if found {
                    let array_close_idx = array_open_idx + relateive_array_close_idx;
                    parsed_path.push(path[array_open_idx..array_close_idx + 1].to_string());
                    current_idx = array_close_idx + 1;
                } else {
                    return Err(ParseError::new("invalid path, no closing array character"));
                }
            },
            (PathElem::ArrayClose, _) => {
                return Err(ParseError::new("invalid path, closing array character before opening"));
            },
            (PathElem::Char, c) => {
                return Err(ParseError::new(&format!("invalid path, found char {}", c)));
            },
            (PathElem::EOW, _) => {
                parsed_path.push(path[current_idx..].to_string());
                break;
            },
        }
    }
    Ok(parsed_path)
}

pub fn traverse(node: &Yaml, head: &str, tail: &[&str], visited: &mut Vec<String>) {
    // if parsed_path still has elements and the node is not a scalar, recurse
    if tail.len() > 0 && !is_scalar(node) {
        recurse(node, tail[0], &tail[1..], visited)
    } else {
        // the parsed path is empty or we have a scalar, try visiting
        visit(node, head, tail, visited);
    }
}

pub fn print_doc_structure(doc: &Yaml) -> EmitResult {
    let out_str = get_node_structure(doc)?;
    debug!("doc structure:\n{}", out_str);
    Ok(())
}

fn is_scalar(node: &Yaml) -> bool {
    match node {
        Yaml::String(_) => true,
        Yaml::Integer(_) => true,
        Yaml::Real(_) => true,
        Yaml::Boolean(_) => true,
        Yaml::Null => true,
        Yaml::BadValue => true,
        _ => false,
    }
}

fn recurse(node: &Yaml, head: &str, tail: &[&str], visited: &mut Vec<String>) {
    // for every entry in the node (we're assuming its a map), traverse if the head matches
    match node {
        Yaml::Hash(h) => {
            for (k, v) in h {
                match k {
                    Yaml::String(k_str) => {
                        if k_str == head {
                            debug!("match on key: {}, traverse", k_str);
                            traverse(v, head, tail, visited);
                        } else {
                            debug!("did not match on key: {}, continue", k_str);
                        }
                    },
                    _ => {
                        error!("key `{:?}` is not a string, exiting", k);
                        std::process::exit(1);
                    },
                }
            }
        },
        Yaml::Array(v) => {
            let array_indices: Vec<usize> = match get_array_idx(head) {
                ArrayIndex:: Star => (0..v.len()).collect(),
                ArrayIndex::Idx(i) => {
                    if i >= v.len() {
                        debug!("array index {} too large, don't recurse", i);
                        return;
                    }
                    vec![i]
                }
            };
            debug!("match on array indices: {:?}, traverse", array_indices);
            for array_idx in array_indices {
                traverse(&v[array_idx], head, tail, visited);
            }
        }
        Yaml::Alias(_a) => panic!("recursing on aliases not implemented yet"),
        // this can remain a panic as it's not yet implemented
        _ => {
            error!("can only recurse on maps, array, or aliases. recursing on `{:?}` is not supported, continuing", node);
        },
    }
}

enum ArrayIndex {
    Star,
    Idx(usize),
}

fn get_array_idx(bracketed_path_elem: &str) -> ArrayIndex {
    if !bracketed_path_elem.starts_with('[') || !bracketed_path_elem.ends_with(']') {
        error!("key `{:?}` is not a valid array index, exiting", bracketed_path_elem);
        std::process::exit(1);
    }
    let path_elem = &bracketed_path_elem[1..bracketed_path_elem.len() - 1];
    if path_elem == "*" {
        return ArrayIndex::Star;
    }
    match path_elem.parse::<usize>() {
        Ok(i) => ArrayIndex::Idx(i),
        Err(e) => {
            error!("unable to parse array index `{:?}`, error: {:?}", path_elem, e);
            std::process::exit(1);
        },
    }
}

fn visit(node: &Yaml, _head: &str, tail: &[&str], visited: &mut Vec<String>) {
    if tail.len() == 0 {
        debug!("tail length is 0, visiting leaf node {:?}", node);
        match node {
            Yaml::String(s) => {
                visited.push(s.to_owned());
            },
            Yaml::Integer(i) => {
                visited.push(i.to_string());
            },
            Yaml::Real(f) => {
                visited.push(f.to_string());
            },
            Yaml::Boolean(b) => {
                visited.push(b.to_string());
            },
            h @ Yaml::Hash(_) => {
                let s = get_node_structure(h).unwrap_or_else(|err| {
                    error!("failed to parse map value `{:?}`: {}", h, err);
                    std::process::exit(1);
                });
                visited.push(s);
            },
            Yaml::Null => {
                visited.push("null".to_string());
            },
            Yaml::BadValue => {
                error!("visited node `{:?}` is a corrupted value, continuing", node);
            },
            v @ Yaml::Array(_) => {
                let s = get_node_structure(v).unwrap_or_else(|err| {
                    error!("failed to parse array value `{:?}`: {}", v, err);
                    std::process::exit(1);
                });
                visited.push(s);
            },
            _a @ Yaml::Alias(_) => {
                panic!("alias type node yet implemented");
            },
        }
        return
    }
    debug!("tail length is not 0, not visiting node {:?}", node);
}

pub fn get_node_structure(node: &Yaml) -> Result<String, EmitError> {
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(node)?;
    // remove initial four characters ("---\n") from the node
    if out_str.len() < 4 {
        error!("invalid node structure `{:?}`", node);
        std::process::exit(1);
    }
    Ok(out_str.trim_start_matches("---\n").to_string())
}

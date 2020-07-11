use crate::path::{
    matches_pattern, parse_array_child_filter, parse_array_indexing_operation, ArrayIndices,
    ParseError, SPLAT,
};
use log::{debug, error};
use yaml_rust::Yaml;

#[derive(Debug)]
pub struct VisitedNode<'a> {
    pub yml: &'a Yaml,
    pub path: String,
}

fn unwrap(s: &str) -> &str {
    if s.len() < 2 {
        return "";
    }
    &s[1..s.len() - 1]
}

// TODO(wdeuschle): test the splat and empty case
// TODO(wdeuchle): test that the handle functions are called properly, handle their errors
fn get_array_idx<F, G>(
    path_elem: &str,
    array_node: &Vec<Yaml>,
    is_final_path_elem: bool,
    handle_child_filter: F,
    handle_indexing_operation: G,
) -> ArrayIndices
where
    F: FnOnce(&str, &Vec<Yaml>, bool) -> Result<ArrayIndices, ParseError>,
    G: FnOnce(&str) -> Result<ArrayIndices, ParseError>,
{
    debug!("getting array index for path_elem: {}", path_elem);
    match path_elem {
        SPLAT => {
            debug!("found splat for array, using all indices");
            ArrayIndices::Star
        }
        _ if path_elem.starts_with('(') && path_elem.ends_with(')') => {
            let path_elem = unwrap(path_elem);
            handle_child_filter(path_elem, array_node, is_final_path_elem).unwrap_or_else(|err| {
                error!("{}", err);
                std::process::exit(1);
            })
        }
        _ if path_elem.starts_with('[') && path_elem.ends_with(']') => {
            let path_elem = unwrap(path_elem);
            handle_indexing_operation(path_elem).unwrap_or_else(|err| {
                error!("{}", err);
                std::process::exit(1);
            })
        }
        _ => {
            debug!(
                "key `{:?}` is neither a valid array index nor child filter, continuing",
                path_elem
            );
            ArrayIndices::Indices(vec![])
        }
    }
}

// TODO: audit for integration testing
pub fn traverse<'a>(
    node: &'a Yaml,
    head: &str,
    tail: &[String],
    path: String,
    following_splat: bool,
    visited: &mut Vec<VisitedNode<'a>>,
) {
    // handle following a splat
    if following_splat {
        if head == SPLAT {
            if tail.len() > 0 {
                // first traversal after finding a splat
                recurse(node, &tail[0], &tail[1..], path, true, visited)
            } else {
                // final path element was a splat
                if is_scalar(node) {
                    visit(node, head, tail, path, visited);
                } else {
                    recurse(node, head, tail, path, false, visited);
                }
            }
        } else if !is_scalar(node) {
            // recurse until you find a non-splat match
            recurse(node, head, tail, path, true, visited);
        }
        return;
    }

    // if parsed_path still has elements and the node is not a scalar, recurse
    if tail.len() > 0 && !is_scalar(node) {
        recurse(node, &tail[0], &tail[1..], path, false, visited)
    } else {
        // the parsed path is empty or we have a scalar, try visiting
        visit(node, head, tail, path, visited);
    }
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

// TODO: does this belong here? maybe a filter package?
fn is_child_filter_value_match(v: &Yaml, p: &str, is_final_path_elem: bool) -> bool {
    if !is_final_path_elem || !(p.starts_with('(') && p.ends_with(')')) {
        return false;
    }
    let p = &p[1..p.len() - 1];
    let filter_key_and_value = crate::path::split_child_filter(p).unwrap_or_else(|err| {
        error!("unable to parse child value filter, error: {:?}", err);
        std::process::exit(1);
    });
    let v_str = &crate::convert::convert_single_node(v);
    let filter_value = filter_key_and_value[1];
    matches_pattern(v_str, filter_value)
}

// TODO(wdeuschle): unit test
fn recurse<'a>(
    node: &'a Yaml,
    head: &str,
    tail: &[String],
    path: String,
    following_splat: bool,
    visited: &mut Vec<VisitedNode<'a>>,
) {
    // for every entry in the node (we're assuming its a map), traverse if the head matches
    match node {
        Yaml::Hash(h) => {
            for (k, v) in h {
                match k {
                    Yaml::String(k_str) => {
                        if following_splat {
                            // traverse deeper, still following a splat
                            debug!("following splat in map for key: {}, traverse", k_str);
                            let mut new_path = path.clone();
                            if new_path.len() > 0 {
                                new_path.push_str(".");
                            }
                            new_path.push_str(k_str);
                            traverse(v, head, tail, new_path, true, visited);
                        }
                        if matches_pattern(k_str, head) {
                            debug!("match on key: {}, traverse", k_str);
                            let mut new_path = path.clone();
                            if new_path.len() > 0 {
                                new_path.push_str(".");
                            }
                            new_path.push_str(k_str);
                            traverse(v, head, tail, new_path, head == SPLAT, visited);
                        } else if is_child_filter_value_match(v, head, tail.len() == 0) {
                            debug!("match on child value filter: {}", head);
                            let mut new_path = path.clone();
                            if new_path.len() > 0 {
                                new_path.push_str(".");
                            }
                            new_path.push_str(k_str);
                            traverse(v, head, tail, new_path, false, visited);
                        } else {
                            debug!("did not match on key: {}, continue", k_str);
                        }
                    }
                    _ => {
                        error!("key `{:?}` is not a string, exiting", k);
                        std::process::exit(1);
                    }
                }
            }
        }
        Yaml::Array(v) => {
            if following_splat {
                // traverse deeper, still following a splat
                debug!("following splat in array, traverse all {} indices", v.len());
                let array_indices: Vec<usize> = (0..v.len()).collect();
                for array_idx in array_indices {
                    let mut new_path = path.clone();
                    new_path.push_str(&format!(".[{}]", array_idx));
                    traverse(&v[array_idx], head, tail, new_path, true, visited);
                }
            }
            let array_indices: Vec<usize> = match get_array_idx(
                head,
                v,
                tail.len() == 0,
                parse_array_child_filter,
                parse_array_indexing_operation,
            ) {
                ArrayIndices::Star => (0..v.len()).collect(),
                ArrayIndices::Indices(indices) => {
                    for i in indices.iter() {
                        if *i >= v.len() {
                            debug!("array index {} too large, don't recurse", i);
                            return;
                        }
                    }
                    indices
                }
            };
            debug!("match on array indices: {:?}, traverse", array_indices);
            for array_idx in array_indices {
                let mut new_path = path.clone();
                new_path.push_str(&format!(".[{}]", array_idx));
                traverse(&v[array_idx], head, tail, new_path, head == SPLAT, visited);
            }
        }
        Yaml::Alias(_a) => panic!("recursing on aliases not implemented yet"),
        // this can remain a panic as it's not yet implemented
        _ => {
            error!("can only recurse on maps, array, or aliases. recursing on `{:?}` is not supported, continuing", node);
        }
    }
}

// TODO(wdeuschle): unit test
fn visit<'a>(
    node: &'a Yaml,
    _head: &str,
    tail: &[String],
    path: String,
    visited: &mut Vec<VisitedNode<'a>>,
) {
    if tail.len() == 0 {
        debug!("tail length is 0, visiting leaf node {:?}", node);
        match node {
            s @ Yaml::String(_) => {
                visited.push(VisitedNode {
                    yml: s,
                    path: path.clone(),
                });
            }
            i @ Yaml::Integer(_) => {
                visited.push(VisitedNode {
                    yml: i,
                    path: path.clone(),
                });
            }
            f @ Yaml::Real(_) => {
                visited.push(VisitedNode {
                    yml: f,
                    path: path.clone(),
                });
            }
            b @ Yaml::Boolean(_) => {
                visited.push(VisitedNode {
                    yml: b,
                    path: path.clone(),
                });
            }
            h @ Yaml::Hash(_) => {
                visited.push(VisitedNode {
                    yml: h,
                    path: path.clone(),
                });
            }
            n @ Yaml::Null => {
                visited.push(VisitedNode {
                    yml: n,
                    path: path.clone(),
                });
            }
            b @ Yaml::BadValue => {
                visited.push(VisitedNode {
                    yml: b,
                    path: path.clone(),
                });
            }
            v @ Yaml::Array(_) => {
                visited.push(VisitedNode {
                    yml: v,
                    path: path.clone(),
                });
            }
            _a @ Yaml::Alias(_) => {
                panic!("alias type node yet implemented");
            }
        }
        return;
    }
    debug!("tail length is not 0, not visiting node {:?}", node);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_child_filter_value_match_not_final_elem() {
        assert!(!is_child_filter_value_match(
            &Yaml::String("crabby".to_string()),
            "",
            false
        ));
    }

    #[test]
    fn test_is_child_filter_value_match_not_a_filter() {
        assert!(!is_child_filter_value_match(
            &Yaml::String("crabby".to_string()),
            "[.==crabby]",
            true
        ));
        assert!(!is_child_filter_value_match(
            &Yaml::String("crabby".to_string()),
            "[.==crabby)",
            true
        ));
        assert!(!is_child_filter_value_match(
            &Yaml::String("crabby".to_string()),
            "(.==crabby]",
            true
        ));
    }

    #[test]
    fn test_is_child_filter_value_match_not_a_match() {
        assert!(!is_child_filter_value_match(
            &Yaml::String("crabby".to_string()),
            "(.==nope)",
            true
        ));
    }

    #[test]
    fn test_is_child_filter_value_match_is_a_match() {
        assert!(is_child_filter_value_match(
            &Yaml::String("crabby".to_string()),
            "(.==crabby)",
            true
        ));
        assert!(is_child_filter_value_match(
            &Yaml::String("crabby".to_string()),
            "(.==crab*)",
            true
        ));
    }

    #[test]
    fn get_array_idx_splat() {
        assert_eq!(
            ArrayIndices::Star,
            get_array_idx(
                "**",
                &vec![Yaml::Null],
                false,
                parse_array_child_filter,
                parse_array_indexing_operation
            )
        );
    }

    #[test]
    fn get_array_idx_invalid_path_elem() {
        assert_eq!(
            ArrayIndices::Indices(vec![]),
            get_array_idx(
                "crabby",
                &vec![Yaml::Null],
                false,
                parse_array_child_filter,
                parse_array_indexing_operation
            )
        );
    }

    #[test]
    fn test_unwrap() {
        let s1 = "";
        assert_eq!("", unwrap(s1));

        let s2 = "(crabby)";
        assert_eq!("crabby", unwrap(s2));

        let s3 = "[crabby]";
        assert_eq!("crabby", unwrap(s3));
    }
}

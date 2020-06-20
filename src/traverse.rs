use crate::print;
use log::{debug, error};
use yaml_rust::Yaml;

pub fn traverse(node: &Yaml, head: &str, tail: &[&str], visited: &mut Vec<String>) {
    // if parsed_path still has elements and the node is not a scalar, recurse
    if tail.len() > 0 && !is_scalar(node) {
        recurse(node, tail[0], &tail[1..], visited)
    } else {
        // the parsed path is empty or we have a scalar, try visiting
        visit(node, head, tail, visited);
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
                    }
                    _ => {
                        error!("key `{:?}` is not a string, exiting", k);
                        std::process::exit(1);
                    }
                }
            }
        }
        Yaml::Array(v) => {
            let array_indices: Vec<usize> = match get_array_idx(head) {
                ArrayIndex::Star => (0..v.len()).collect(),
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
        }
    }
}

enum ArrayIndex {
    Star,
    Idx(usize),
}

fn get_array_idx(bracketed_path_elem: &str) -> ArrayIndex {
    if !bracketed_path_elem.starts_with('[') || !bracketed_path_elem.ends_with(']') {
        error!(
            "key `{:?}` is not a valid array index, exiting",
            bracketed_path_elem
        );
        std::process::exit(1);
    }
    let path_elem = &bracketed_path_elem[1..bracketed_path_elem.len() - 1];
    if path_elem == "*" {
        return ArrayIndex::Star;
    }
    match path_elem.parse::<usize>() {
        Ok(i) => ArrayIndex::Idx(i),
        Err(e) => {
            error!(
                "unable to parse array index `{:?}`, error: {:?}",
                path_elem, e
            );
            std::process::exit(1);
        }
    }
}

fn visit(node: &Yaml, _head: &str, tail: &[&str], visited: &mut Vec<String>) {
    if tail.len() == 0 {
        debug!("tail length is 0, visiting leaf node {:?}", node);
        match node {
            Yaml::String(s) => {
                visited.push(s.to_owned());
            }
            Yaml::Integer(i) => {
                visited.push(i.to_string());
            }
            Yaml::Real(f) => {
                visited.push(f.to_string());
            }
            Yaml::Boolean(b) => {
                visited.push(b.to_string());
            }
            h @ Yaml::Hash(_) => {
                let s = print::get_node_structure(h).unwrap_or_else(|err| {
                    error!("failed to parse map value `{:?}`: {}", h, err);
                    std::process::exit(1);
                });
                visited.push(s);
            }
            Yaml::Null => {
                visited.push("null".to_string());
            }
            Yaml::BadValue => {
                error!("visited node `{:?}` is a corrupted value, continuing", node);
            }
            v @ Yaml::Array(_) => {
                let s = print::get_node_structure(v).unwrap_or_else(|err| {
                    error!("failed to parse array value `{:?}`: {}", v, err);
                    std::process::exit(1);
                });
                visited.push(s);
            }
            _a @ Yaml::Alias(_) => {
                panic!("alias type node yet implemented");
            }
        }
        return;
    }
    debug!("tail length is not 0, not visiting node {:?}", node);
}

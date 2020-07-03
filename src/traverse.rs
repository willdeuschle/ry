use log::{debug, error};
use yaml_rust::Yaml;

enum ArrayIndex {
    Star,
    Idx(usize),
}

// TODO(wdeuschle): unit test
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

pub fn traverse<'a>(node: &'a Yaml, head: &str, tail: &[&str], visited: &mut Vec<&'a Yaml>) {
    // if parsed_path still has elements and the node is not a scalar, recurse
    if tail.len() > 0 && !is_scalar(node) {
        recurse(node, tail[0], &tail[1..], visited)
    } else {
        // the parsed path is empty or we have a scalar, try visiting
        visit(node, head, tail, visited);
    }
}

// TODO(wdeuschle): unit test
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

// TODO(wdeuschle): unit test
fn key_matches_path(k: &str, p: &str) -> bool {
    if k == p {
        return true;
    }
    if p.ends_with('*') {
        let truncated_p = p.trim_end_matches('*');
        if k.starts_with(truncated_p) {
            return true;
        }
    }
    false
}

// TODO(wdeuschle): unit test
fn recurse<'a>(node: &'a Yaml, head: &str, tail: &[&str], visited: &mut Vec<&'a Yaml>) {
    // for every entry in the node (we're assuming its a map), traverse if the head matches
    match node {
        Yaml::Hash(h) => {
            for (k, v) in h {
                match k {
                    Yaml::String(k_str) => {
                        if key_matches_path(k_str, head) {
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

// TODO(wdeuschle): unit test
fn visit<'a>(node: &'a Yaml, _head: &str, tail: &[&str], visited: &mut Vec<&'a Yaml>) {
    if tail.len() == 0 {
        debug!("tail length is 0, visiting leaf node {:?}", node);
        match node {
            s @ Yaml::String(_) => {
                visited.push(s);
            }
            i @ Yaml::Integer(_) => {
                visited.push(i);
            }
            f @ Yaml::Real(_) => {
                visited.push(f);
            }
            b @ Yaml::Boolean(_) => {
                visited.push(b);
            }
            h @ Yaml::Hash(_) => {
                visited.push(h);
            }
            n @ Yaml::Null => {
                visited.push(n);
            }
            b @ Yaml::BadValue => {
                visited.push(b);
            }
            v @ Yaml::Array(_) => {
                visited.push(v);
            }
            _a @ Yaml::Alias(_) => {
                panic!("alias type node yet implemented");
            }
        }
        return;
    }
    debug!("tail length is not 0, not visiting node {:?}", node);
}

use log::{debug, error};
use yaml_rust::Yaml;

const SPLAT: &'static str = "**";

#[derive(Debug)]
pub struct VisitedNode<'a> {
    pub yml: &'a Yaml,
    pub path: String,
}

enum ArrayIndices {
    Star,
    Indices(Vec<usize>),
}

// TODO(wdeuschle): unit test
fn get_array_idx_for_child_filter(
    path_elem: &str,
    array_node: &Vec<Yaml>,
    is_final_path_elem: bool,
) -> ArrayIndices {
    if path_elem == "*" {
        return ArrayIndices::Star;
    }
    let filter_key_and_value = crate::path::parse_child_filter(path_elem).unwrap_or_else(|err| {
        error!("unable to parse child filter, error: {:?}", err);
        std::process::exit(1);
    });
    let (filter_path, filter_value) = (filter_key_and_value[0], filter_key_and_value[1]);

    // parse filter_path
    let parsed_path = crate::path::parse_path_into(filter_path);
    debug!("parsed path for child filtering: {:?}", parsed_path);

    let mut indices: Vec<usize> = vec![];

    if is_final_path_elem {
        // child value filter
        debug!("running a child value filter");
        for (idx, array_elem) in array_node.iter().enumerate() {
            if key_matches_path(
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
            if key_matches_path(
                &crate::convert::convert_single_node(visited_elem.yml),
                filter_value, // path element for child filter
            ) {
                debug!("array_elem matched child node filter: {:?}", array_elem);
                indices.push(idx);
            }
        }
        debug!("child node filtering matched indices: {:?}", indices);
    }
    return ArrayIndices::Indices(indices);
}

// TODO(wdeuschle): unit test
fn get_array_idx(
    path_elem: &str,
    array_node: &Vec<Yaml>,
    is_final_path_elem: bool,
) -> ArrayIndices {
    debug!("getting array index for path_elem: {}", path_elem);
    if path_elem == SPLAT {
        debug!("found splat for array, using all indices");
        return ArrayIndices::Star;
    }
    if path_elem.starts_with('(') && path_elem.ends_with(')') {
        let path_elem = &path_elem[1..path_elem.len() - 1];
        return get_array_idx_for_child_filter(path_elem, array_node, is_final_path_elem);
    } else if !path_elem.starts_with('[') || !path_elem.ends_with(']') {
        debug!(
            "key `{:?}` is neither a valid array index nor child filter, continuing",
            path_elem
        );
        return ArrayIndices::Indices(vec![]);
    }
    let path_elem = &path_elem[1..path_elem.len() - 1];
    if path_elem == "*" {
        return ArrayIndices::Star;
    }
    match path_elem.parse::<usize>() {
        Ok(i) => ArrayIndices::Indices(vec![i]),
        Err(e) => {
            error!(
                "unable to parse array index `{:?}`, error: {:?}",
                path_elem, e
            );
            std::process::exit(1);
        }
    }
}

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

// TODO(wdeuschle): unit test, could also use a rename
fn key_matches_path(k: &str, p: &str) -> bool {
    if k == p || p == SPLAT {
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
fn is_child_filter_value_match(v: &Yaml, p: &str, is_final_path_elem: bool) -> bool {
    if !is_final_path_elem || !(p.starts_with('(') && p.ends_with(')')) {
        return false;
    }
    let p = &p[1..p.len() - 1];
    let filter_key_and_value = crate::path::parse_child_filter(p).unwrap_or_else(|err| {
        error!("unable to parse child value filter, error: {:?}", err);
        std::process::exit(1);
    });
    let v_str = &crate::convert::convert_single_node(v);
    let filter_value = filter_key_and_value[1];
    key_matches_path(v_str, filter_value)
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
                        if key_matches_path(k_str, head) {
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
            let array_indices: Vec<usize> = match get_array_idx(head, v, tail.len() == 0) {
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

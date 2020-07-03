use log::{debug, error};
use yaml_rust::emitter::{EmitError, EmitResult};
use yaml_rust::{Yaml, YamlEmitter};

pub fn print_doc_structure(doc: &Yaml) -> EmitResult {
    let out_str = get_node_structure(doc)?;
    debug!("doc structure:\n{}", out_str);
    Ok(())
}

// TODO(wdeuschle): integration test
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

// TODO(wdeuschle): needs individual testing?
pub fn parse_single_node(node: &Yaml) -> String {
    match node {
        Yaml::String(s) => {
            format!("{}", s)
        }
        Yaml::Integer(i) => {
            format!("{}", i)
        }
        Yaml::Real(f) => {
            format!("{}", f)
        }
        Yaml::Boolean(b) => {
            format!("{}", b)
        }
        h @ Yaml::Hash(_) => {
            let s = get_node_structure(h).unwrap_or_else(|err| {
                error!("failed to parse map value `{:?}`: {}", h, err);
                std::process::exit(1);
            });
            format!("{}", s)
        }
        Yaml::Null => {
            format!("null")
        }
        Yaml::BadValue => {
            format!("node `{:?}` is corrupted", node)
        }
        v @ Yaml::Array(_) => {
            let s = get_node_structure(v).unwrap_or_else(|err| {
                error!("failed to parse array value `{:?}`: {}", v, err);
                std::process::exit(1);
            });
            format!("{}", s)
        }
        _a @ Yaml::Alias(_) => {
            panic!("alias type node yet implemented");
        }
    }
}

// TODO(wdeuschle): needs testing
pub fn print_length(node: &Yaml) {
    match node {
        Yaml::String(s) => {
            println!("{}", s.len());
        }
        Yaml::Hash(h) => {
            println!("{}", h.len());
        }
        Yaml::Array(a) => {
            println!("{}", a.len());
        }
        Yaml::Integer(i) => {
            println!("{}", i.to_string().len());
        }
        Yaml::Real(f) => {
            println!("{}", f.to_string().len());
        }
        Yaml::Boolean(b) => {
            println!("{}", b.to_string().len());
        }
        Yaml::Null => {
            println!("0");
        }
        Yaml::BadValue => {
            println!("node `{:?}` is corrupted", node);
        }
        _a @ Yaml::Alias(_) => {
            panic!("alias type node yet implemented");
        }
    }
}

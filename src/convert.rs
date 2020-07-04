use log::{debug, error};
use yaml_rust::emitter::{EmitError, EmitResult};
use yaml_rust::{Yaml, YamlEmitter};

pub fn debug_print_doc_structure(doc: &Yaml) -> EmitResult {
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
pub fn convert_single_node(node: &Yaml) -> String {
    match node {
        Yaml::String(s) => format!("{}", s),
        Yaml::Integer(i) => format!("{}", i),
        Yaml::Real(f) => format!("{}", f),
        Yaml::Boolean(b) => format!("{}", b),
        h @ Yaml::Hash(_) => {
            let s = get_node_structure(h).unwrap_or_else(|err| {
                error!("failed to convert map value `{:?}` to string: {}", h, err);
                std::process::exit(1);
            });
            format!("{}", s)
        }
        Yaml::Null => format!("null"),
        Yaml::BadValue => format!("node `{:?}` is corrupted", node),
        v @ Yaml::Array(_) => {
            let s = get_node_structure(v).unwrap_or_else(|err| {
                error!("failed to convert array value `{:?}` to string: {}", v, err);
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
pub fn convert_length(node: &Yaml) -> String {
    match node {
        Yaml::String(s) => format!("{}", s.len()),
        Yaml::Hash(h) => format!("{}", h.len()),
        Yaml::Array(a) => format!("{}", a.len()),
        Yaml::Integer(i) => format!("{}", i.to_string().len()),
        Yaml::Real(f) => format!("{}", f.to_string().len()),
        Yaml::Boolean(b) => format!("{}", b.to_string().len()),
        Yaml::Null => format!("0"),
        Yaml::BadValue => format!("node `{:?}` is corrupted", node),
        _a @ Yaml::Alias(_) => panic!("alias type node yet implemented"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust::YamlLoader;

    #[test]
    fn test_convert_length_string() {
        let s = Yaml::String("four".to_string());
        assert_eq!(convert_length(&s), "4");
    }

    #[test]
    fn test_convert_length_hash() {
        let hash_str = "
a:
  item_b
b:
  item_c
c:
  item_d";
        let hash = &YamlLoader::load_from_str(&hash_str).unwrap()[0];
        assert_eq!(convert_length(&hash), "3");
    }

    #[test]
    fn test_convert_length_array() {
        let array_str = "
- a
- b
- c";
        let array = &YamlLoader::load_from_str(&array_str).unwrap()[0];
        assert_eq!(convert_length(&array), "3");
    }

    #[test]
    fn test_convert_length_integer() {
        assert_eq!(convert_length(&Yaml::Integer(100)), "3");
    }

    #[test]
    fn test_convert_length_real() {
        assert_eq!(convert_length(&Yaml::Real(".001".to_string())), "4");
    }

    #[test]
    fn test_convert_length_boolean() {
        assert_eq!(convert_length(&Yaml::Boolean(true)), "4");
    }

    #[test]
    fn test_convert_length_null() {
        assert_eq!(convert_length(&Yaml::Null), "0");
    }
}

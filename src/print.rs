use log::{debug, error};
use yaml_rust::emitter::{EmitError, EmitResult};
use yaml_rust::{Yaml, YamlEmitter};

pub fn print_doc_structure(doc: &Yaml) -> EmitResult {
    let out_str = get_node_structure(doc)?;
    debug!("doc structure:\n{}", out_str);
    Ok(())
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

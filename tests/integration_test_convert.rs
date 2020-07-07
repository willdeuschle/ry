use ry::{convert_length, convert_single_node};
use yaml_rust::{Yaml, YamlLoader};

#[test]
fn test_convert_single_node() {
    assert_eq!(
        convert_single_node(&Yaml::String("string".to_string())),
        "string"
    );
    assert_eq!(convert_single_node(&Yaml::Integer(1)), "1");
    assert_eq!(convert_single_node(&Yaml::Real(0.01.to_string())), "0.01");
    assert_eq!(convert_single_node(&Yaml::Boolean(true)), "true");
    let hash_str = "a: b";
    let hash = &YamlLoader::load_from_str(hash_str).unwrap()[0];
    match hash {
        Yaml::Hash(_) => {}
        _ => panic!("invalid, not hash type"),
    };
    assert_eq!(convert_single_node(hash), hash_str);
    let array_str = "- a";
    let array = &YamlLoader::load_from_str(array_str).unwrap()[0];
    match array {
        Yaml::Array(_) => {}
        _ => panic!("invalid, not array type"),
    };
    assert_eq!(convert_single_node(array), array_str);
    assert_eq!(convert_single_node(&Yaml::Null), "null");
}

#[test]
fn test_convert_length() {
    assert_eq!(convert_length(&Yaml::String("four".to_string())), "4");

    let hash_str = "
a:
item_b
b:
item_c
c:
item_d";
    let hash = &YamlLoader::load_from_str(&hash_str).unwrap()[0];
    assert_eq!(convert_length(&hash), "3");

    let array_str = "
- a
- b
- c";
    let array = &YamlLoader::load_from_str(&array_str).unwrap()[0];
    assert_eq!(convert_length(&array), "3");

    assert_eq!(convert_length(&Yaml::Integer(100)), "3");

    assert_eq!(convert_length(&Yaml::Real(".001".to_string())), "4");

    assert_eq!(convert_length(&Yaml::Boolean(true)), "4");

    assert_eq!(convert_length(&Yaml::Null), "0");
}

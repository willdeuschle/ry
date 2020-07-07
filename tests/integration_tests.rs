use ry::convert::{convert_length, convert_single_node};
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

#[test]
fn test_parse_path() {
    assert_eq!(ry::parse_path("a.b.c").unwrap(), vec!["a", "b", "c"]);
}

#[test]
fn test_parse_path_with_quotes() {
    assert_eq!(
        ry::parse_path("a.\"foo.bar\".c").unwrap(),
        vec!["a", "foo.bar", "c"]
    );
}

#[test]
fn test_parse_path_with_one_quote_fails() {
    let result = ry::parse_path("a.\"foo.bar.c");
    let expected = Err(ry::ParseError::new("invalid path, no closing quote"));
    assert_eq!(result, expected);
}

#[test]
fn test_parse_path_with_array_indexing() {
    assert_eq!(
        ry::parse_path("a.foo[10].bar").unwrap(),
        vec!["a", "foo", "[10]", "bar"]
    );
}

#[test]
fn test_parse_path_with_parens() {
    assert_eq!(
        ry::parse_path("a.(b.d==cat*).c").unwrap(),
        vec!["a", "(b.d==cat*)", "c"]
    );
}

#[test]
fn test_parse_path_with_one_open_array_panics() {
    let result = ry::parse_path("a.(b.d==cat*.c");
    let expected = Err(ry::ParseError::new(
        "invalid path, no closing paren character",
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_parse_path_with_one_open_paren_panics() {
    let result = ry::parse_path("a.foo[1.bar");
    let expected = Err(ry::ParseError::new(
        "invalid path, no closing array character",
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_parse_path_with_child_value_filtering() {
    assert_eq!(
        ry::parse_path("animals(.==cat)").unwrap(),
        vec!["animals", "(.==cat)"]
    );
}

#[test]
fn test_traverse_leaf() {
    let docs_str = "
a:
  b:
    c: 2";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["a".to_string(), "b".to_string(), "c".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 1);
    assert_eq!(convert_single_node(visited[0].yml), "2");
}

#[test]
fn test_traverse_non_leaf() {
    let docs_str = "
a:
  b:
    c: 2";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["a".to_string(), "b".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 1);
    assert_eq!(convert_single_node(visited[0].yml), "c: 2");
}

#[test]
fn test_traverse_with_quoted_key() {
    let docs_str = "
a:
  foo.bar:
    c: 2";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["a".to_string(), "foo.bar".to_string(), "c".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 1);
    assert_eq!(convert_single_node(visited[0].yml), "2");
}

#[test]
fn test_traverse_array() {
    let docs_str = "
a:
  b:
    - 1
    - 2
    - 3";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["a".to_string(), "b".to_string(), "[1]".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 1);
    assert_eq!(convert_single_node(visited[0].yml), "2");
}

#[test]
fn test_traverse_array_wildcard() {
    let docs_str = "
a:
  b:
    - 1
    - 2
    - 3";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["a".to_string(), "b".to_string(), "[*]".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 3);
    assert_eq!(convert_single_node(visited[0].yml), "1");
    assert_eq!(convert_single_node(visited[1].yml), "2");
    assert_eq!(convert_single_node(visited[2].yml), "3");
}

#[test]
fn test_traverse_array_after_index() {
    let docs_str = "
a:
  b:
    - 1
    - 2
    - c: d";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec![
            "a".to_string(),
            "b".to_string(),
            "[*]".to_string(),
            "c".to_string(),
        ],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 1);
    assert_eq!(convert_single_node(visited[0].yml), "d");
}

#[test]
fn test_traverse_hash_prefix_match() {
    let docs_str = "
a:
  item_b:
    f: 1
  thing_c:
    f: 2
  item_d:
    f: 3
  thing_e:
    f: 4";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["a".to_string(), "item*".to_string(), "f".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 2);
    assert_eq!(convert_single_node(visited[0].yml), "1");
    assert_eq!(convert_single_node(visited[1].yml), "3");
}

#[test]
fn test_traverse_hash_wildcard() {
    let docs_str = "
a:
  item_b:
    f: 1
  thing_c:
    f: 2
  item_d:
    f: 3
  thing_e:
    f: 4";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["a".to_string(), "*".to_string(), "f".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 4);
    assert_eq!(convert_single_node(visited[0].yml), "1");
    assert_eq!(convert_single_node(visited[1].yml), "2");
    assert_eq!(convert_single_node(visited[2].yml), "3");
    assert_eq!(convert_single_node(visited[3].yml), "4");
}

#[test]
fn test_child_array_filtering() {
    let docs_str = "
a:
  - b:
      c: thing0
      d: leopard
    ba: fast
  - b:
      c: thing1 # MATCHES
      d: cat
    ba: meowy
  - b:
      c: thing2
      d: caterpillar
    ba: icky
  - b:
      c: thing3 # MATCHES
      d: cat
    ba: also meowy";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec![
            "a".to_string(),
            "(b.d==cat)".to_string(),
            "b".to_string(),
            "c".to_string(),
        ],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 2);
    assert_eq!(convert_single_node(visited[0].yml), "thing1");
    assert_eq!(convert_single_node(visited[1].yml), "thing3");
}

#[test]
fn test_child_array_filtering_with_wildcard() {
    let docs_str = "
a:
  - b:
      c: thing0
      d: leopard
    ba: fast
  - b:
      c: thing1 # MATCHES
      d: cat
    ba: meowy
  - b:
      c: thing2 # MATCHES
      d: caterpillar
    ba: icky
  - b:
      c: thing3 # MATCHES
      d: cat
    ba: also meowy";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec![
            "a".to_string(),
            "(b.d==cat*)".to_string(),
            "b".to_string(),
            "c".to_string(),
        ],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 3);
    assert_eq!(convert_single_node(visited[0].yml), "thing1");
    assert_eq!(convert_single_node(visited[1].yml), "thing2");
    assert_eq!(convert_single_node(visited[2].yml), "thing3");
}

#[test]
fn test_handle_splat() {
    let docs_str = "
a:
  b1:
    c: # MATCHES
      c: thing1 # MATCHES
    d: cat cat
  b2:
    c: thing2 # MATCHES
    d: dog dog
  b3:
    d:
    - f:
        c: thing3 # MATCHES
        d: beep
    - f:
        g:
          c: thing4 # MATCHES
          d: boop
    - d: mooo";

    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["a".to_string(), "**".to_string(), "c".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 5);
    assert_eq!(convert_single_node(visited[0].yml), "thing1");
    assert_eq!(convert_single_node(visited[1].yml), "c: thing1");
    assert_eq!(convert_single_node(visited[2].yml), "thing2");
    assert_eq!(convert_single_node(visited[3].yml), "thing3");
    assert_eq!(convert_single_node(visited[4].yml), "thing4");
}

#[test]
fn test_handle_splat_ending() {
    let docs_str = "
a:
  b1:
    c:
      c: thing1 # MATCHES
    d: cat cat # MATCHES
  b2:
    c: thing2 # MATCHES
    d: dog dog # MATCHES
  b3:
    d:
    - f:
        c: thing3 # MATCHES
        d: beep # MATCHES
    - f:
        g:
          c: thing4 # MATCHES # MATCHES
          d: boop # MATCHES
    - d: mooo # MATCHES";

    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["a".to_string(), "**".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 9);
    assert_eq!(convert_single_node(visited[0].yml), "thing1");
    assert_eq!(convert_single_node(visited[1].yml), "cat cat");
    assert_eq!(convert_single_node(visited[2].yml), "thing2");
    assert_eq!(convert_single_node(visited[3].yml), "dog dog");
    assert_eq!(convert_single_node(visited[4].yml), "thing3");
    assert_eq!(convert_single_node(visited[5].yml), "beep");
    assert_eq!(convert_single_node(visited[6].yml), "thing4");
    assert_eq!(convert_single_node(visited[7].yml), "boop");
    assert_eq!(convert_single_node(visited[8].yml), "mooo");
}

#[test]
fn test_handle_child_value_filter_array() {
    let docs_str = "
animals:
  - cats
  - dog
  - cheetah";

    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["animals".to_string(), "(.==c*)".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 2);
    assert_eq!(convert_single_node(visited[0].yml), "cats");
    assert_eq!(convert_single_node(visited[1].yml), "cheetah");
}

#[test]
fn test_handle_child_value_filter_map() {
    let docs_str = "
animals:
  cats: yes
  dogs: yessiree
  lions: nope
  cheetas:
    but: yes";

    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited = Vec::<ry::VisitedNode>::new();
    ry::traverse(
        &doc,
        "",
        &vec!["animals".to_string(), "(.==yes*)".to_string()],
        String::new(),
        false,
        &mut visited,
    );
    assert_eq!(visited.len(), 2);
    assert_eq!(convert_single_node(visited[0].yml), "yes");
    assert_eq!(convert_single_node(visited[1].yml), "yessiree");
}

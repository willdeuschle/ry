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
fn test_parse_path_with_one_open_array_panics() {
    let result = ry::parse_path("a.foo[1.bar");
    let expected = Err(ry::ParseError::new(
        "invalid path, no closing array character",
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_traverse_leaf() {
    use yaml_rust::YamlLoader;

    let docs_str = "
a:
  b:
    c: 2";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited: Vec<String> = Vec::new();
    ry::traverse(&doc, "", &vec!["a", "b", "c"], &mut visited);
    assert_eq!(visited.len(), 1);
    assert_eq!(visited[0], "2");
}

#[test]
fn test_traverse_non_leaf() {
    use yaml_rust::YamlLoader;

    let docs_str = "
a:
  b:
    c: 2";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited: Vec<String> = Vec::new();
    ry::traverse(&doc, "", &vec!["a", "b"], &mut visited);
    assert_eq!(visited.len(), 1);
    assert_eq!(visited[0], "c: 2");
}

#[test]
fn test_traverse_with_quoted_key() {
    use yaml_rust::YamlLoader;

    let docs_str = "
a:
  foo.bar:
    c: 2";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited: Vec<String> = Vec::new();
    ry::traverse(&doc, "", &vec!["a", "foo.bar", "c"], &mut visited);
    assert_eq!(visited.len(), 1);
    assert_eq!(visited[0], "2");
}

#[test]
fn test_traverse_array() {
    use yaml_rust::YamlLoader;

    let docs_str = "
a:
  b:
    - 1
    - 2
    - 3";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited: Vec<String> = Vec::new();
    ry::traverse(&doc, "", &vec!["a", "b", "[1]"], &mut visited);
    assert_eq!(visited.len(), 1);
    assert_eq!(visited[0], "2");
}

#[test]
fn test_traverse_array_wildcard() {
    use yaml_rust::YamlLoader;

    let docs_str = "
a:
  b:
    - 1
    - 2
    - 3";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited: Vec<String> = Vec::new();
    ry::traverse(&doc, "", &vec!["a", "b", "[*]"], &mut visited);
    assert_eq!(visited.len(), 3);
    assert_eq!(visited[0], "1");
    assert_eq!(visited[1], "2");
    assert_eq!(visited[2], "3");
}

#[test]
fn test_traverse_array_after_index() {
    use yaml_rust::YamlLoader;

    let docs_str = "
a:
  b:
    - 1
    - 2
    - c: d";
    let doc = &YamlLoader::load_from_str(&docs_str).unwrap()[0];

    let mut visited: Vec<String> = Vec::new();
    ry::traverse(&doc, "", &vec!["a", "b", "[*]", "c"], &mut visited);
    assert_eq!(visited.len(), 1);
    assert_eq!(visited[0], "d");
}

#[test]
fn test_traverse_hash_prefix_match() {
    use yaml_rust::YamlLoader;

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

    let mut visited: Vec<String> = Vec::new();
    ry::traverse(&doc, "", &vec!["a", "item*", "f"], &mut visited);
    assert_eq!(visited.len(), 2);
    assert_eq!(visited[0], "1");
    assert_eq!(visited[1], "3");
}

#[test]
fn test_traverse_hash_wildcard() {
    use yaml_rust::YamlLoader;

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

    let mut visited: Vec<String> = Vec::new();
    ry::traverse(&doc, "", &vec!["a", "*", "f",], &mut visited);
    assert_eq!(visited.len(), 4);
    assert_eq!(visited[0], "1");
    assert_eq!(visited[1], "2");
    assert_eq!(visited[2], "3");
    assert_eq!(visited[3], "4");
}

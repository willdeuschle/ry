use ry::{parse_path, ParseError};

#[test]
fn test_parse_path() {
    assert_eq!(parse_path("a.b.c").unwrap(), vec!["a", "b", "c"]);
}

#[test]
fn test_parse_path_with_quotes() {
    assert_eq!(
        parse_path("a.\"foo.bar\".c").unwrap(),
        vec!["a", "foo.bar", "c"]
    );
}

#[test]
fn test_parse_path_with_one_quote_fails() {
    let result = parse_path("a.\"foo.bar.c");
    let expected = Err(ParseError::new("invalid path, no closing quote"));
    assert_eq!(result, expected);
}

#[test]
fn test_parse_path_with_array_indexing() {
    assert_eq!(
        parse_path("a.foo[10].bar").unwrap(),
        vec!["a", "foo", "[10]", "bar"]
    );
}

#[test]
fn test_parse_path_with_parens() {
    assert_eq!(
        parse_path("a.(b.d==cat*).c").unwrap(),
        vec!["a", "(b.d==cat*)", "c"]
    );
}

#[test]
fn test_parse_path_with_one_open_array_panics() {
    let result = parse_path("a.(b.d==cat*.c");
    let expected = Err(ParseError::new("invalid path, no closing paren character"));
    assert_eq!(result, expected);
}

#[test]
fn test_parse_path_with_one_open_paren_panics() {
    let result = parse_path("a.foo[1.bar");
    let expected = Err(ParseError::new("invalid path, no closing array character"));
    assert_eq!(result, expected);
}

#[test]
fn test_parse_path_with_child_value_filtering() {
    assert_eq!(
        parse_path("animals(.==cat)").unwrap(),
        vec!["animals", "(.==cat)"]
    );
}

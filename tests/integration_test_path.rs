use ry::parse_path;

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
fn test_parse_path_with_one_quote_errs() {
    let result = parse_path("a.\"foo.bar.c");
    assert_eq!(true, result.is_err());
    assert_eq!(
        true,
        format!("{}", result.unwrap_err()).ends_with("no closing quote")
    );
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
fn test_parse_path_with_one_open_array_errs() {
    let result = parse_path("a.foo[1.bar");
    assert_eq!(true, result.is_err());
    assert_eq!(
        true,
        format!("{}", result.unwrap_err()).ends_with("no closing array character")
    );
}

#[test]
fn test_parse_path_with_one_open_paren_errs() {
    let result = parse_path("a.(b.d==cat*.c");
    assert_eq!(true, result.is_err());
    assert_eq!(
        true,
        format!("{}", result.unwrap_err()).ends_with("no closing paren character")
    );
}

#[test]
fn test_parse_path_with_open_array_start_errs() {
    let result = parse_path("a.foo]1].bar");
    assert_eq!(true, result.is_err());
    assert_eq!(
        true,
        format!("{}", result.unwrap_err()).ends_with("closing array character before opening")
    );
}

#[test]
fn test_parse_path_with_close_paren_start_errs() {
    let result = parse_path("a.)b.d==cat*.c)");
    assert_eq!(true, result.is_err());
    assert_eq!(
        true,
        format!("{}", result.unwrap_err()).ends_with("closing paren character before opening")
    );
}

#[test]
fn test_parse_path_with_child_value_filtering() {
    assert_eq!(
        parse_path("animals(.==cat)").unwrap(),
        vec!["animals", "(.==cat)"]
    );
}

// TODO(wdeuschle): support child map filtering (like array filtering on child elems, but for
// maps). note this is not part of yq
// NOTE(wdeuschle): merge keys not yet supported in yaml_rust: https://github.com/chyh1990/yaml-rust/issues/68
fn main() {
    ry::cli::run_cli();
}

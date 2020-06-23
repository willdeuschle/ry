use clap::{App, Arg};
use log::{debug, error, LevelFilter, Metadata, Record};
use std::io::{self, Read};
use yaml_rust::{Yaml, YamlLoader};

static LOGGER: SimpleLogger = SimpleLogger;

// TODO(wdeuschle): add unit tests to modules
// TODO(wdeuschle): implement remaining visit edge cases -> just alias
// TODO(wdeuschle): at some point, we need to stream the yaml tokens instead of reading the file all
// at once
fn main() {
    let matches = App::new("ry")
        .version("0.0")
        .author("Will Deuschle")
        .about("structured search in yaml files")
        .arg(
            Arg::with_name("yaml_file")
                .help("sets the input yaml file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("path_expression")
                .help("path to search against")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("default_value")
                .takes_value(true)
                .help("default value to print if there are no matching nodes")
                .long("defaultValue"),
        )
        .arg(
            Arg::with_name("doc_idx")
                .takes_value(true)
                .help("document index to search")
                .long("docIndex")
                .short("d"),
        )
        .arg(
            Arg::with_name("debug")
                .help("enable debug logging")
                .long("debug")
                .short("v"),
        )
        .get_matches();

    let file_name = matches.value_of("yaml_file").unwrap();
    let path = matches.value_of("path_expression").unwrap();

    let log_level = if matches.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };
    let _ = log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log_level))
        .unwrap_or_else(|err| {
            eprintln!("failed to set logger: `{}`", err);
        });

    let docs_str = if file_name == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .unwrap_or_else(|err| {
                error!("failed to read from stdin: `{}`", err);
                std::process::exit(1);
            });
        buffer
    } else {
        std::fs::read_to_string(file_name).unwrap_or_else(|err| {
            error!("failed to read file `{}`: `{}`", file_name, err);
            std::process::exit(1);
        })
    };
    let mut docs: &[Yaml] = &YamlLoader::load_from_str(&docs_str).unwrap_or_else(|err| {
        error!("failed to load yaml file `{}`: `{}`", file_name, err);
        std::process::exit(1);
    });

    // Multi document support, doc is a yaml::Yaml
    if docs.len() == 0 {
        error!("no yaml documents found in file `{}`", file_name);
        std::process::exit(1);
    } else if matches.is_present("doc_idx") {
        let doc_idx = matches.value_of("doc_idx").unwrap();
        if doc_idx == "*" {
            debug!(
                "processing all `{}` documents in file `{}`",
                docs.len(),
                file_name
            );
        } else {
            let parsed_doc_idx = match doc_idx.parse::<usize>() {
                Ok(idx) => idx,
                Err(e) => {
                    error!(
                        "failed to parse document index `{}`, error: {:?}",
                        doc_idx, e
                    );
                    std::process::exit(1);
                }
            };
            if parsed_doc_idx >= docs.len() {
                error!("only `{}` documents are present in file `{}`, but document index `{}` was requested for searching",
                    docs.len(), file_name, parsed_doc_idx);
                std::process::exit(1);
            }
            debug!(
                "processing document at index `{}` in file `{}`",
                parsed_doc_idx, file_name
            );
            docs = &docs[parsed_doc_idx..parsed_doc_idx + 1]
        }
    } else {
        debug!(
            "processing all `{}` documents in file `{}`",
            docs.len(),
            file_name
        );
    }

    for ref doc in docs {
        if log_level == LevelFilter::Debug {
            ry::print_doc_structure(doc).unwrap_or_else(|err| {
                error!(
                    "unable to print display document from file `{}`: {}",
                    file_name, err
                );
                std::process::exit(1);
            });
        }

        // parse path
        // TODO: can this be cleaned up?
        let parsed_path_res = ry::parse_path(path);
        let parsed_path_vec: Vec<String> = match parsed_path_res {
            Ok(_) => parsed_path_res.unwrap(),
            Err(e) => {
                error!("failed to parse path, error: {}", e);
                std::process::exit(1);
            }
        };
        let parsed_path: Vec<&str> = parsed_path_vec.iter().map(String::as_str).collect();
        debug!("parsed path: {:?}", parsed_path);

        let mut visited = Vec::<String>::new();
        ry::traverse(doc, "", &parsed_path, &mut visited);
        if visited.len() == 0 && matches.is_present("default_value") {
            let dv = matches.value_of("default_value").unwrap();
            debug!("found no matches, using default value `{}`", dv);
            visited.push(dv.to_string());
        }
        debug!("matched values: {:?}", visited);
        for value in visited {
            println!("{}", value);
        }
    }
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

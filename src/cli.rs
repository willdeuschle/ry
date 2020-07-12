use clap::{App, Arg};
use log::{debug, error, LevelFilter, Metadata, Record};
use std::io::{self, Read};
use yaml_rust::{Yaml, YamlLoader};

use crate::{
    convert_length, convert_single_node, debug_print_doc_structure, parse_path, traverse,
    VisitedNode,
};

static LOGGER: SimpleLogger = SimpleLogger;

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

#[derive(Debug)]
enum PrintMode {
    Value,
    Path,
    ValueAndPath,
}

fn parse_print_mode(mode: &str) -> PrintMode {
    match mode {
        "v" => PrintMode::Value,
        "p" => PrintMode::Path,
        "pv" => PrintMode::ValueAndPath,
        "vp" => PrintMode::ValueAndPath,
        _ => PrintMode::Value,
    }
}

pub fn run_cli() {
    let yaml_file_arg = "yaml_file";
    let path_expression_arg = "yaml_file";
    let default_value_arg = "default_value";
    let length_arg = "length";
    let print_mode_arg = "print_mode";
    let collect_arg = "collect";
    let doc_idx_arg = "doc_idx";
    let debug_arg = "debug";

    let matches = App::new("ry")
        .version("0.0")
        .author("Will Deuschle")
        .about("structured search in yaml files")
        .arg(
            Arg::with_name(yaml_file_arg)
                .help("sets the input yaml file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name(path_expression_arg)
                .help("path to search against")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name(default_value_arg)
                .takes_value(true)
                .help("default value to print if there are no matching nodes")
                .long("defaultValue"),
        )
        .arg(
            Arg::with_name(length_arg)
                .help("prints length of results")
                .long("length")
                .short("L"),
        )
        .arg(
            Arg::with_name(print_mode_arg)
                .takes_value(true)
                .help("what mode to print results in")
                .long("printMode")
                .short("p"),
        )
        .arg(
            Arg::with_name(collect_arg)
                .takes_value(false)
                .help("collect results into an array")
                .long("collect")
                .short("C"),
        )
        .arg(
            Arg::with_name(doc_idx_arg)
                .takes_value(true)
                .help("document index to search")
                .long("docIndex")
                .short("d"),
        )
        .arg(
            Arg::with_name(debug_arg)
                .help("enable debug logging")
                .long("debug")
                .short("v"),
        )
        .get_matches();

    let file_name = matches.value_of(yaml_file_arg).unwrap();
    let path = matches.value_of(path_expression_arg).unwrap();

    let log_level = if matches.is_present(debug_arg) {
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
    } else if matches.is_present(doc_idx_arg) {
        let doc_idx = matches.value_of(doc_idx_arg).unwrap();
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
            debug_print_doc_structure(doc).unwrap_or_else(|err| {
                error!(
                    "unable to print display document from file `{}`: {}",
                    file_name, err
                );
                std::process::exit(1);
            });
        }

        // parse path
        let parsed_path = match parse_path(path) {
            Ok(parsed_path) => parsed_path,
            Err(e) => {
                error!("failed to parse path, error: {}", e);
                std::process::exit(1);
            }
        };
        debug!("parsed path: {:?}", parsed_path);

        let mut visited = Vec::<VisitedNode>::new();
        traverse(doc, "", &parsed_path, String::new(), false, &mut visited);

        let default_yml: Yaml;
        let default_visited_node: VisitedNode;
        if visited.len() == 0 && matches.is_present(default_value_arg) {
            let dv = matches.value_of(default_value_arg).unwrap();
            debug!("found no matches, using default value `{}`", dv);
            default_yml = Yaml::from_str(dv);
            default_visited_node = VisitedNode {
                yml: &default_yml,
                path: "".to_string(),
            };
            visited.push(default_visited_node);
        }
        debug!("matched values: {:?}", visited);

        let print_mode = parse_print_mode(matches.value_of(print_mode_arg).unwrap_or("v"));
        debug!("print_mode: {:?}", print_mode);

        let collect = matches.is_present(collect_arg);
        debug!("collect: {}", collect);

        if matches.is_present(length_arg) {
            // length mode
            if collect {
                // length and collect just prints the number of visited nodes
                println!("{}", visited.len());
            } else {
                match print_mode {
                    PrintMode::Path => {
                        for value in visited {
                            println!("{}", value.path);
                        }
                    }
                    PrintMode::Value => {
                        for value in visited {
                            println!("{}", convert_length(value.yml));
                        }
                    }
                    PrintMode::ValueAndPath => {
                        for value in visited {
                            println!("{}: {}", value.path, convert_length(value.yml));
                        }
                    }
                }
            }
        } else {
            // no length mode
            let collect_prepend = if collect { "- " } else { "" };
            match print_mode {
                PrintMode::Path => {
                    for value in visited {
                        println!("{}{}", collect_prepend, value.path);
                    }
                }
                PrintMode::Value => {
                    for value in visited {
                        println!("{}{}", collect_prepend, convert_single_node(value.yml));
                    }
                }
                PrintMode::ValueAndPath => {
                    for value in visited {
                        println!(
                            "{}{}: {}",
                            collect_prepend,
                            value.path,
                            convert_single_node(value.yml)
                        );
                    }
                }
            }
        }
    }
}

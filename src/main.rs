use yaml_rust::YamlLoader;
use clap::{Arg, App};
use log::{debug, error, Record, Metadata, LevelFilter};

static LOGGER: SimpleLogger = SimpleLogger;

// TODO(wdeuschle): implement remaining visit edge cases -> just alias
// TODO(wdeuschle): at some point, we need to stream the yaml tokens instead of reading the file all
// at once
// TODO(wdeuschle): add multi-doc support
// TODO(wdeuschle): add regex support
// TODO(wdeuschle): add stdin support
// TODO(wdeuschle): breakk library into logical modules
fn main() {
    let matches = App::new("ry")
                          .version("0.0")
                          .author("Will Deuschle")
                          .about("structured search in yaml files")
                          .arg(Arg::with_name("yaml_file")
                               .help("sets the input yaml file to use")
                               .required(true)
                               .index(1))
                          .arg(Arg::with_name("path_expression")
                               .help("path to search against")
                               .required(true)
                               .index(2))
                          .arg(Arg::with_name("debug")
                               .help("enable debug logging")
                               .long("debug")
                               .short("d"))
                          .get_matches();

    let file_name = matches.value_of("yaml_file").unwrap();
    let path = matches.value_of("path_expression").unwrap();

    let log_level = if matches.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(log_level)).unwrap_or_else(|err| {
        eprintln!("failed to set logger: `{}`", err);
    });

    let docs_str = std::fs::read_to_string(file_name).unwrap_or_else(|err| {
        error!("failed to read file `{}`: `{}`", file_name, err);
        std::process::exit(1);
    });
    let docs = YamlLoader::load_from_str(&docs_str).unwrap_or_else(|err| {
        error!("failed to load yaml file `{}`: `{}`", file_name, err);
        std::process::exit(1);
    });

    // Multi document support, doc is a yaml::Yaml
    if docs.len() == 0 {
        error!("no yaml documents found in file `{}`", file_name);
        std::process::exit(1);
    } else if docs.len() > 1 {
        debug!("found more than one yaml document in file `{}`, only processing first", file_name);
    }
    let doc = &docs[0];

    if log_level == LevelFilter::Debug {
        ry::print_doc_structure(doc).unwrap_or_else(|err| {
            error!("unable to print display document from file `{}`: {}", file_name, err);
            std::process::exit(1);
        });
    }

    // parse path
    // TODO: can this be cleaned up?
    let parsed_path_res = ry::parse_path(path);
    let parsed_path_vec: Vec<String> = match parsed_path_res {
        Ok(_) => parsed_path_res.unwrap(),
        Err(e) => {
            error!("failed to parse path: {}", e);
            std::process::exit(1);
        },
    };
    let parsed_path: Vec<&str> = parsed_path_vec.iter().map(String::as_str).collect();
    debug!("parsed path: {:?}", parsed_path);

    let mut visited = Vec::<String>::new();
    ry::traverse(doc, "", &parsed_path, &mut visited);
    debug!("matched values: {:?}", visited);
    for value in visited {
        println!("{}", value);
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

use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::DirEntry;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .version("0.1.0")
        .author("Kashif Yousuf <kashifyousuf.sc@gmail.com>")
        .about("find in Rust")
        .arg(
            Arg::with_name("paths")
                .help("Search Path(s)")
                .value_name("PATH")
                .multiple(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name("names")
                .help("NAME")
                .value_name("NAME")
                .short("n")
                .long("name")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("entry_types")
                .help("entry_types")
                .value_name("Entry Types")
                .short("t")
                .long("type")
                .takes_value(true)
                .multiple(true)
                .possible_values(&["d", "f", "l"]),
        )
        .get_matches();

    let entry_types = matches
        .values_of_lossy("entry_types")
        .map(|types| {
            types
                .iter()
                .map(|t| match t.as_str() {
                    "d" => Dir,
                    "f" => File,
                    "l" => Link,
                    _ => unreachable!("Not found"),
                })
                .collect()
        })
        .unwrap_or_default();

    let names = matches
        .values_of_lossy("names")
        .map(|names| {
            names
                .into_iter()
                .map(|name| Regex::new(&name).map_err(|_| format!("Invalid --name \"{}\"", name)))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        names,
        entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    for path in config.paths {
        let name_filter = |entry: &DirEntry| {
            config.entry_types.is_empty()
                || config
                    .entry_types
                    .iter()
                    .any(|entry_type| match entry_type {
                        Link => entry.file_type().is_symlink(),
                        Dir => entry.file_type().is_dir(),
                        File => entry.file_type().is_file(),
                    })
        };

        let type_filter = |entry: &DirEntry| {
            config.names.is_empty()
                || config
                    .names
                    .iter()
                    .any(|re| re.is_match(&entry.file_name().to_string_lossy()))
        };

        let entries = WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| match entry {
                Err(e) => {
                    eprintln!("{e}");
                    None
                }
                Ok(entry) => Some(entry),
            })
            .filter(name_filter)
            .filter(type_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();

        println!("{}", entries.join("\n"));
    }

    Ok(())
}

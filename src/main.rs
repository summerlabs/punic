extern crate futures;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate serde;
extern crate shellexpand;
extern crate tokio;

use crate::punfile::parse_pun_file;
use crate::utils::download::download_dependencies;
use crate::utils::upload::upload_dependencies;
use clap::{App, Arg};
use std::borrow::Borrow;

mod cache;
mod punfile;
mod utils;

const OVERRIDE_DEPENDENCIES_COMMAND: &str = "OVERRIDE_DEPENDENCIES";
const IGNORE_LOCAL_CACHE: &str = "IGNORE_LOCAL_CACHE";
const IGNORE_OUTPUT_CACHE: &str = "IGNORE_OUTPUT_CACHE";
const CACHE_PREFIX: &str = "CACHE_PREFIX";

#[tokio::main]
async fn main() {
    let matches = App::new("Punic Carthage")
        .version("1.0.0")
        .about("ios dependency caching made great again")
        .author("Johnson Cheung")
        .subcommand(
            App::new("download")
                .about("scan your punfile and download dependencies")
                .arg(
                    Arg::with_name(crate::IGNORE_LOCAL_CACHE)
                        .short("l")
                        .long("ignore-local")
                        .help("ignore the local cache and download anyway then copy")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name(crate::IGNORE_OUTPUT_CACHE)
                        .short("o")
                        .long("ignore-output")
                        .help("ignore the output cache and copy anyway")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name(crate::OVERRIDE_DEPENDENCIES_COMMAND)
                        .short("d")
                        .long("dependencies")
                        .multiple(true)
                        .allow_hyphen_values(true)
                        .value_delimiter(" ")
                        .value_terminator(";"),
                )
                .arg(
                    Arg::with_name(crate::CACHE_PREFIX)
                        .short("p")
                        .long("cache-prefix")
                        .value_name(crate::CACHE_PREFIX)
                        .help("set custom prefix for directory")
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("upload")
                .about("upload to s3")
                .arg(
                    Arg::with_name(crate::IGNORE_LOCAL_CACHE)
                        .short("l")
                        .long("ignore-local")
                        .help("ignore the local cache and zip anyway")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name(crate::OVERRIDE_DEPENDENCIES_COMMAND)
                        .short("d")
                        .long("dependencies")
                        .multiple(true)
                        .allow_hyphen_values(true)
                        .value_delimiter(";"),
                )
                .arg(
                    Arg::with_name(crate::CACHE_PREFIX)
                        .short("p")
                        .long("cache-prefix")
                        .value_name(crate::CACHE_PREFIX)
                        .help("set custom prefix for directory")
                        .takes_value(true),
                ),
        )
        .get_matches();

    let punfile = parse_pun_file(matches.borrow());
    let local_cache = punfile.configuration.local.clone();

    let cache_dir = shellexpand::tilde(local_cache.as_str());
    let output_dir = format!("{}/build/{}", cache_dir, punfile.configuration.prefix);

    std::fs::create_dir_all(output_dir).unwrap();

    // create Carthage build path if it does not exist
    std::fs::create_dir_all(punfile.configuration.output.clone()).unwrap();

    if let Some(ref matches) = matches.subcommand_matches("download") {
        download_dependencies(punfile, matches, cache_dir).await;
    } else if let Some(ref matches) = matches.subcommand_matches("upload") {
        upload_dependencies(punfile, matches, cache_dir).await;
    }
}

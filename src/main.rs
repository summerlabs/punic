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

mod cache;
mod punfile;
mod utils;

#[tokio::main]
async fn main() {
    let matches = App::new("Punic Carthage")
        .version("0.0.7")
        .about("ios dependency caching made great again")
        .author("Johnson Cheung")
        .arg(
            Arg::with_name("CachePrefix")
                .short("p")
                .long("cache-prefix")
                .value_name("Cache Prefix")
                .help("set custom prefix for directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ForceCommand")
                .short("f")
                .long("force")
                .value_name("Force Command")
                .help("force the command ignoring cache")
                .takes_value(true),
        )
        .subcommand(
            App::new("download")
                .about("scan your punfile and download dependencies")
                .arg(
                    Arg::with_name("ForceCommand")
                        .short("f")
                        .long("force")
                        .value_name("Force Command")
                        .help("force the command ignoring cache")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dependencies")
                        .short("d")
                        .long("deps")
                        .multiple(true)
                        .allow_hyphen_values(true)
                        .value_delimiter(" ")
                        .value_terminator(";"),
                ),
        )
        .subcommand(
            App::new("upload")
                .about("upload to s3")
                .arg(
                    Arg::with_name("ForceCommand")
                        .short("f")
                        .long("force")
                        .value_name("Force Command")
                        .help("force the command ignoring cache")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dependencies")
                        .short("d")
                        .long("deps")
                        .multiple(true)
                        .allow_hyphen_values(true)
                        .value_delimiter(";"),
                ),
        )
        .get_matches();

    let punfile = parse_pun_file(matches.clone());
    let local_cache = punfile.configuration.local.clone();

    let expanded_str = shellexpand::tilde(local_cache.as_str());

    let output_dir = format!("{}/build/{}", expanded_str, punfile.configuration.prefix);
    std::fs::create_dir_all(output_dir).unwrap();

    // create Carthage build path if it does not exist
    std::fs::create_dir_all(punfile.configuration.output.clone()).unwrap();

    if let Some(ref matches) = matches.subcommand_matches("download") {
        download_dependencies(punfile, matches, expanded_str).await;
    } else if let Some(ref matches) = matches.subcommand_matches("upload") {
        upload_dependencies(punfile, matches, expanded_str).await;
    }
}

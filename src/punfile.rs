use crate::punfile;
use crate::punfile::data::{Configuration, PunFile, Repository};
use clap::ArgMatches;
use serde_yaml::Value;

pub mod data {

    pub struct Configuration {
        pub prefix: String,
        pub local: String,
        pub s3_bucket: String,
    }

    pub struct PunFile {
        pub configuration: Configuration,
        pub frameworks: Vec<Repository>,
    }

    pub struct Repository {
        pub repo_name: String,
        pub name: String,
    }
}

pub fn parse_pun_file(matches: ArgMatches) -> punfile::data::PunFile {
    let contents = std::fs::read_to_string("Punfile")
        .expect("Unable to read Punfile, make sure one exists in your project.");
    let contents_yaml: serde_yaml::Value = serde_yaml::from_str(contents.as_str()).unwrap();
    let cache = contents_yaml
        .get("configuration")
        .expect("Unable to read key `configuration` in Punfile.");
    let default_prefix = &Value::String("output".into());
    let prefix = cache
        .get("prefix")
        .unwrap_or(default_prefix)
        .as_str()
        .unwrap_or("output");
    let local = cache
        .get("local")
        .unwrap()
        .as_str()
        .unwrap_or("~/Library/Caches/Punic");

    let s3_bucket = cache
        .get("s3Bucket")
        .expect("Unable to read key `s3Bucket` in Punfile.")
        .as_str()
        .unwrap();

    let cache_prefix = matches
        .value_of("CachePrefix")
        .unwrap_or(String::from(prefix).as_str())
        .to_string();

    let mut punfile = PunFile {
        configuration: Configuration {
            prefix: cache_prefix,
            local: String::from(local),
            s3_bucket: String::from(s3_bucket),
        },
        frameworks: Vec::new(),
    };

    println!("Cache Prefix: {}", punfile.configuration.prefix);
    println!("Cache Local Path: {}", punfile.configuration.local);
    println!("S3 Bucket: {}", punfile.configuration.s3_bucket);

    let repository_map = contents_yaml
        .get("dependencies")
        .expect("Unable to read key `dependencies` in Punfile.")
        .as_sequence()
        .expect("Key `dependencies` in Punfile must be an array");

    for repo in repository_map {
        let name = repo.as_mapping().unwrap();
        for (key, value) in name.iter() {
            let repo_name = key.as_str().unwrap();
            for seq in value.as_sequence().unwrap().iter() {
                let map_name = seq
                    .as_mapping()
                    .unwrap()
                    .get(&serde_yaml::Value::from("name"));
                let repository = Repository {
                    repo_name: String::from(repo_name),
                    name: String::from(map_name.unwrap().as_str().unwrap()),
                };
                punfile.frameworks.push(repository);
            }
        }
    }
    return punfile;
}

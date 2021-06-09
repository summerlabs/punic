use crate::punfile;
use crate::punfile::data::{Configuration, PunFile, Repository};
use clap::ArgMatches;
use serde_yaml::Value;

pub mod data {

    pub struct Configuration {
        pub prefix: String,
        pub local: String,
        pub output: String,
        pub s3_bucket: String,
    }

    pub struct PunFile {
        pub configuration: Configuration,
        pub frameworks: Vec<Repository>,
    }

    pub struct Repository {
        pub repo_name: String,
        pub name: String,
        pub version: String
    }
}

pub fn parse_pun_file(matches: &ArgMatches) -> punfile::data::PunFile {
    let contents = std::fs::read_to_string("Punfile")
        .expect("Unable to read Punfile, make sure one exists in your project.");
    let contents_yaml: serde_yaml::Value = serde_yaml::from_str(contents.as_str()).unwrap();
    let configuration = contents_yaml
        .get("configuration")
        .expect("Unable to read key `configuration` in Punfile.");
    let prefix = get_cache_prefix(matches, configuration);
    let local = configuration
        .get("local")
        .expect("Unable to read key `local` in Punfile.")
        .as_str()
        .unwrap_or("~/Library/Caches/Punic");
    let output = configuration
        .get("output")
        .expect("Unable to read key `output` in Punfile.")
        .as_str()
        .unwrap_or("Carthage/Build");
    let s3_bucket = configuration
        .get("s3Bucket")
        .expect("Unable to read key `s3Bucket` in Punfile.")
        .as_str()
        .unwrap();

    let mut punfile = PunFile {
        configuration: Configuration {
            prefix,
            local: String::from(local),
            output: String::from(output),
            s3_bucket: String::from(s3_bucket),
        },
        frameworks: Vec::new(),
    };

    println!("Cache Prefix\t\t: {}", punfile.configuration.prefix);
    println!("Cache Local Path\t: {}", punfile.configuration.local);
    println!("Cache Output Path\t: {}", punfile.configuration.output);
    println!("S3 Bucket\t\t: {}", punfile.configuration.s3_bucket);

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
                let version = seq.as_mapping().unwrap().get(&serde_yaml::Value::from("version"));
                let repository = Repository {
                    repo_name: String::from(repo_name),
                    name: String::from(map_name.unwrap().as_str().unwrap()),
                    version: String::from(version.unwrap_or(&serde_yaml::Value::String("".into())).as_str().unwrap())
                };
                punfile.frameworks.push(repository);
            }
        }
    }
    return punfile;
}

fn get_cache_prefix(matches: &ArgMatches, configuration: &Value) -> String {
    let default_prefix = &Value::String("output".into());
    let punfile_prefix = configuration
        .get("prefix")
        .unwrap_or(default_prefix)
        .as_str()
        .unwrap_or("output");
    if let Some(ref matches) = matches.subcommand_matches("download") {
        matches
            .value_of(crate::CACHE_PREFIX)
            .unwrap_or(punfile_prefix)
            .to_string()
    } else if let Some(ref matches) = matches.subcommand_matches("upload") {
        matches
            .value_of(crate::CACHE_PREFIX)
            .unwrap_or(punfile_prefix)
            .to_string()
    } else {
        punfile_prefix.to_string()
    }
}


pub fn print_pun_deps(punfile: &PunFile) {
    let frameworks = &punfile.frameworks;

    for framework in frameworks {
        println!("group: {} , artifact: {}, version: {}", 
            framework.repo_name, 
            framework.name, 
            framework.version);
    }

}


extern crate tokio;
extern crate rusoto_core;
extern crate serde;
extern crate shellexpand;
extern crate rusoto_s3;
extern crate futures;
use std::env;
use std::io::Read;
use std::io::BufReader;
use std::fs::File;
use std::panic;
use std::thread;
use serde_yaml;
use rusoto_core::{Region,ByteStream,RusotoError,RusotoResult};
use rusoto_s3::{GetObjectRequest,PutObjectRequest, S3, S3Client, StreamingBody,GetObjectOutput,GetObjectError};
use zip::write::{FileOptions, ZipWriter};
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use tokio::io::{self,AsyncReadExt, AsyncWrite,AsyncWriteExt};
use zip;
use futures::stream::TryStreamExt;
use futures::FutureExt;
use futures::future::join_all;
use std::io::{Seek, Write};
use std::future::Future;
use clap::{App, Arg, Values};
use std;
use punfile::data::{Repository,CacheSetting,PunFile};
use std::process::{Command, Stdio};
use serde_yaml::Value;
use std::ops::Deref;

mod punfile;
mod cache;
mod utils;

const CARTHAGE_BUILD: &str = "Carthage/Build";
const OUTPUT: &str = "Carthage/Output";



fn parse_pun_file() -> punfile::data::PunFile {
    let contents = std::fs::read_to_string("Punfile").expect("something went wrong with reading file");
    let d: serde_yaml::Value = serde_yaml::from_str(contents.as_str()).unwrap();
    let cache = d.get("cache").unwrap();

    let defaultPrefix = &Value::String("output".into());

    let prefix = cache.get("prefix").unwrap_or(defaultPrefix).as_str().unwrap_or("output");
    let local = cache.get("local").unwrap().as_str().unwrap_or("~/Library/Caches/Punic");
    let s3_bucket = cache.get("s3Bucket").unwrap().as_str().unwrap();
    println!("Cache Prefix: {}", prefix);
    println!("Cache Local Path: {}", local);
    println!("S3 Bucket: {}", s3_bucket);
    let mut pun_file = PunFile {
        cache: CacheSetting {
            prefix: String::from(prefix),
            local: String::from(local),
            s3_bucket: String::from(s3_bucket)
        },
        frameworks: Vec::new()
    };
    let repository_map = d.get("repositoryMap").unwrap().as_sequence().unwrap();
    for  repo in repository_map {
        let name = repo.as_mapping().unwrap();
        for (key,value) in name.iter(){
            let repo_name = key.as_str().unwrap();
            for seq in value.as_sequence().unwrap().iter(){
                let map_name = seq.as_mapping().unwrap().get(&serde_yaml::Value::from("name"));
                let repository = Repository{
                    repo_name: String::from(repo_name),
                    name: String::from(map_name.unwrap().as_str().unwrap())
                };
                pun_file.frameworks.push(repository);
            }
        }
    }
    return pun_file;
}

fn scan_xcframeworks() -> Vec<String>{
    println!("Scanning frameworks in Carthage build folder...");
    let mut frameworks = vec![];
    for entry in fs::read_dir(CARTHAGE_BUILD).unwrap(){
        let en = entry.unwrap();
        let path = en.path();
        if path.is_dir() && path.to_str().unwrap().contains("xcframework") {
            let pathStr = path.to_str().unwrap().to_string().split("/").last().unwrap().to_string();
            println!("{}",pathStr);
            frameworks.push(pathStr);
        }
    }  
    return frameworks;
}


#[tokio::main]
async fn main() {
    let matches = App::new("Punic Carthage")
        .version("1.0")
        .about("ios dependency caching made great again")
       .author("Johnson Cheung")
       .arg(Arg::with_name("CachePrefix")
        .short("p")
        .long("cache-prefix")
        .value_name("Cache Prefix")
        .help("set custom prefix for directory")
        .takes_value(true)
        )
        .arg(Arg::with_name("ForceCommand")
            .short("f")
            .long("force")
            .value_name("Force Command")
            .help("force the command ignoring cache")
            .takes_value(true)
        )
        .subcommand(App::new("download")
            .about("scan your punfile and download dependencies")
            .arg(Arg::with_name("dependencies")
                .short("d")
                .long("deps")
                .multiple(true)
                .allow_hyphen_values(true)
                .value_delimiter(" ")
                .value_terminator(";")
            )
        )
        .subcommand(App::new("upload")
            .about("upload to s3")
            .arg(Arg::with_name("dependencies")
                .short("d")
                .long("deps")
                .multiple(true)
                .allow_hyphen_values(true)
                .value_delimiter(";")
            )

        )
       .get_matches();

    let pun = parse_pun_file();
    let local_cache = pun.cache.local.clone();
    let cache_prefix = matches.value_of("CachePrefix")
        .unwrap_or(pun.cache.prefix.as_str()).to_string();
    println!("cache prefix {}", cache_prefix.clone());
    let force_command = matches.value_of("ForceCommand")
        .unwrap_or("false");

    let expanded_str = shellexpand::tilde(local_cache.as_str());

    let output_dir = format!("{}/build/{}",expanded_str,cache_prefix);
    std::fs::create_dir_all(output_dir).unwrap();

    // create Carthage build path if it does not exist
    std::fs::create_dir_all(CARTHAGE_BUILD).unwrap();
    if let Some(ref matches) = matches.subcommand_matches("download") {
        let mut children = vec![];
        let requested_frameworks:Vec<Repository> = matches.values_of("dependencies").unwrap_or(Values::default()).map(|iter| Repository{ repo_name:String::from(iter), name:String::from(iter)}).collect();
        let mut frameworks = pun.frameworks;
        if(!requested_frameworks.is_empty()) {
            let mut filtered_frameworks:Vec<Repository> = Vec::new();
            for dep in &requested_frameworks {
                let temp = frameworks.iter().find(|item| item.repo_name.eq(&dep.repo_name));
                if(temp.is_none()){
                    println!("{} is not a dependency", dep.repo_name);
                }else{
                    let frame = temp.unwrap();
                    filtered_frameworks.push(Repository {
                        repo_name: frame.repo_name.to_string(),
                        name: frame.name.to_string()
                    });
                }
            }
            frameworks = filtered_frameworks;
        }



        for deps in frameworks {
            let framework_name = format!("{}.xcframework",deps.name);
            let dest_dir = format!("{}/build/{}/{}.xcframework.zip",expanded_str,cache_prefix,deps.name).to_string();
            let src_dir = format!("{}/{}",CARTHAGE_BUILD,deps.name);
            let path = Path::new(dest_dir.as_str());
            let prefix = cache_prefix.clone();
            if path.exists() {
                let dep_path_format = format!("{}/{}.xcframework", CARTHAGE_BUILD, deps.name);
                let dep_path = Path::new(dep_path_format.as_str());
                if !dep_path.exists() {
                    let task = tokio::spawn(async move {
                        utils::archive::extract_zip(CARTHAGE_BUILD,dest_dir.as_str(),framework_name.as_str());
                    });
                    children.push(task);
                } else {
                    println!("Already downloaded {}", path.display());
                }
            } else {
                let s3_bucket = pun.cache.s3_bucket.clone();
                let task = tokio::spawn( async move {
                    cache::s3::download_from_s3(dest_dir.to_string(), prefix.to_string(), s3_bucket).await;
                    let path = Path::new(dest_dir.as_str());
                    if path.exists() {
                        utils::archive::extract_zip(CARTHAGE_BUILD,dest_dir.as_str(),framework_name.as_str());
                    }
                });
                children.push(task);
            }
        }
        join_all(children).await;
    }
    if let Some(ref matches) = matches.subcommand_matches("upload") {
        let mut children = vec![];
        let requested_frameworks:Vec<&str> = matches.values_of("dependencies").unwrap_or(Values::default()).collect();
        let mut frameworks = scan_xcframeworks();
        if(!requested_frameworks.is_empty()) {
            let mut filtered_frameworks:Vec<String> = Vec::new();
            for dep in &requested_frameworks {
                let temp = frameworks.iter().find(|item| item.contains(dep));
                if(temp.is_none()){
                    println!("{} is not a dependency", dep);
                }else{
                    let frame = temp.unwrap();
                    filtered_frameworks.push(frame.to_string());
                }
            }
            frameworks = filtered_frameworks;
        }
        for frame in frameworks {
            println!("Found {}/{}",CARTHAGE_BUILD,frame);
            let src_dir = format!("{}/{}",CARTHAGE_BUILD,frame);
            let dest_dir =  {
                format!("{}/build/{}/{}.zip",expanded_str,cache_prefix,frame)
            };
            let bucket_name = pun.cache.s3_bucket.clone();
            let prefix = cache_prefix.clone();
            if Path::new(&dest_dir).exists() && force_command != "true" {
                println!("Already zipped {}", frame);
                let dest = dest_dir;
                let pref = prefix.to_string();
                let task = tokio::spawn(async move {
                    cache::s3::upload(dest, pref, bucket_name).await;
                });
                children.push(task);
            } else {
                let dest = dest_dir.clone();
                let task = tokio::spawn( async move {
                    let file = File::create(dest).unwrap();
                    let walkdir = WalkDir::new(src_dir.to_string());
                    let it = walkdir.into_iter();
                    utils::archive::zip_dir(&mut it.filter_map(|e| e.ok()), src_dir.as_str(), file, zip::CompressionMethod::DEFLATE);
                    cache::s3::upload(dest_dir,prefix.to_string(),bucket_name).await;
                });
                children.push(task);
            }
        }
        join_all(children).await;
    }
}




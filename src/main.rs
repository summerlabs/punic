
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
use clap::{App,Arg};
use std;
use punfile::data::{Repository,CacheSetting,PunFile};
mod punfile;
mod cache;
mod utils;

const CARTHAGE_BUILD: &str = "Carthage/Build";
const OUTPUT: &str = "Carthage/Output";



fn parse_pun_file() -> punfile::data::PunFile {
    println!("reading file");
    let contents = std::fs::read_to_string("Punfile").expect("something went wrong with reading file");
    let d: serde_yaml::Value = serde_yaml::from_str(contents.as_str()).unwrap();
    let local = d.get("cache").unwrap().get("local").unwrap().as_str();
    let s3_bucket = d.get("cache").unwrap().get("s3Bucket").unwrap().as_str();
    let mut punfile = PunFile {
        cache: CacheSetting {
            local: String::from(local.unwrap()),
            s3_bucket: String::from(s3_bucket.unwrap())
        },
        frameworks: Vec::new() 
    };
    let repositoryMap = d.get("repositoryMap").unwrap().as_sequence().unwrap();
    for  repo in repositoryMap {
        let name = repo.as_mapping().unwrap();
        for (key,value) in name.iter(){
            let repo_name = key.as_str().unwrap();
            for seq in value.as_sequence().unwrap().iter(){
                let map_name = seq.as_mapping().unwrap().get(&serde_yaml::Value::from("name"));
                //let vers = seq.as_mapping().unwrap().get(&serde_yaml::Value::from("version"));
                let repository = Repository{
                    repo_name: String::from(repo_name),
                    //version: String::from(vers.unwrap().as_str().unwrap()),
                    name: String::from(map_name.unwrap().as_str().unwrap()),
                    platforms: Vec::new()
                };
                punfile.frameworks.push(repository);
            }
        }
    } 
    return punfile;
}

fn scan_xcframeworks() -> Vec<String>{
    println!("scan frameworks");
    let mut frameworks = vec![];
    for entry in fs::read_dir(CARTHAGE_BUILD).unwrap(){
        let en = entry.unwrap();
        let path = en.path();
        if path.is_dir() && path.to_str().unwrap().contains("xcframework") {
            let pathStr = path.to_str().unwrap().to_string().split("/").last().unwrap().to_string();
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
        .subcommand(App::new("download").about("scan your punfile and download dependencies"))
        .subcommand(App::new("upload").about("upload to s3"))
       .get_matches();
    let pun = parse_pun_file();
    let cache_prefix = matches.value_of("CachePrefix").unwrap_or("output").to_string();

    let expanded_str = shellexpand::tilde(pun.cache.local.as_str());
    let output_dir = format!("{}/build/{}",expanded_str,cache_prefix);
    std::fs::create_dir_all(output_dir).unwrap();
    
    println!("{}/build/{}", pun.cache.local, cache_prefix);
    if let Some(ref matches) = matches.subcommand_matches("download") {
        for deps in pun.frameworks {
            println!("{}/{}.xcframework.zip",CARTHAGE_BUILD,deps.name);
            let src_dir = format!("{}/{}",CARTHAGE_BUILD,deps.name);
            let dest_dir = format!("{}/build/{}/{}.xcframework.zip",expanded_str,cache_prefix,deps.name);
            let framework_name = format!("{}.xcframework",deps.name);
            let path = Path::new(dest_dir.as_str());
            if( path.exists()){
                utils::archive::extract_zip(CARTHAGE_BUILD,dest_dir.as_str(),framework_name.as_str());
            } else {
                let s3_bucket = pun.cache.s3_bucket.clone();
                let result = cache::s3::download_from_s3(dest_dir.as_str(),cache_prefix.as_str(),s3_bucket).await.unwrap_or_else(|e| {

                });
                let path = Path::new(dest_dir.as_str());
                if(path.exists()) {
                    utils::archive::extract_zip(CARTHAGE_BUILD,dest_dir.as_str(),framework_name.as_str());
                }
            }
        }
    }
    if let Some(ref matches) = matches.subcommand_matches("upload") {
        let frameworks = scan_xcframeworks();
        let mut children = vec![];
        for frame in frameworks {
            let src_dir = format!("{}/{}",CARTHAGE_BUILD,frame);
            let dest_dir =  {
                format!("{}/build/{}/{}.zip",expanded_str,cache_prefix,frame)
            };
            let bucket_name = pun.cache.s3_bucket.clone();
            let prefix = cache_prefix.clone();
            if(Path::new(&dest_dir).exists()) {
                println!("framework {} already zipped", frame);
                let dest = dest_dir;
                let pref = prefix.to_string();

                let task = tokio::spawn(async move {
                    cache::s3::upload(dest, pref, bucket_name).await;
                });
                children.push(task);
            }else{
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




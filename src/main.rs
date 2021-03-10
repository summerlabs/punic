
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
use std::io::{Seek, Write};
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
                let repository = Repository{
                    repo_name: String::from(repo_name),
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

//fn zip_dir<T>(
//    it: &mut dyn Iterator<Item = DirEntry>,
//    prefix: &str,
//    writer: T,
//    method: zip::CompressionMethod,
//) -> zip::result::ZipResult<()>
//where
//    T: Write + Seek,
//{
//    let mut zip = zip::ZipWriter::new(writer);
//    let options = FileOptions::default()
//        .compression_method(method)
//        .unix_permissions(0o755);
//
//    let mut buffer = Vec::new();
//    for entry in it {
//        let path = entry.path();
//        let name = path.strip_prefix(Path::new(prefix)).unwrap();
//
//        // Write file or directory explicitly
//        // Some unzip tools unzip files with directory paths correctly, some do not!
//        if path.is_file() {
//            println!("adding file {:?} as {:?} ...", path, name);
//            #[allow(deprecated)]
//            zip.start_file_from_path(name, options)?;
//            let mut f = File::open(path)?;
//
//            f.read_to_end(&mut buffer)?;
//            zip.write_all(&*buffer)?;
//            buffer.clear();
//        } else if name.as_os_str().len() != 0 {
//            // Only if not root! Avoids path spec / warning
//            // and mapname conversion failed error on unzip
//            println!("adding dir {:?} as {:?} ...", path, name);
//            #[allow(deprecated)]
//            zip.add_directory_from_path(name, options)?;
//        }
//    }
//    zip.finish()?;
//    Result::Ok(())
//}


//fn extract_zip(path: &str,dest: &str){
//
//    let file = fs::File::open(path).unwrap();
//    let mut archive = zip::ZipArchive::new(file).unwrap();
//
//    for i in 0..archive.len() {
//        let mut file = archive.by_index(i).unwrap();
//        let outpath = match file.enclosed_name() {
//            Some(path) => path.to_owned(),
//            None => continue,
//        };
//
//        {
//            let comment = file.comment();
//            if !comment.is_empty() {
//                println!("File {} comment: {}", i, comment);
//            }
//        }
//        
//
//        if (&*file.name()).ends_with('/') {
//            println!("File {} extracted to \"{}\"", i, outpath.display());
//            let output = format!("{}/{}/{}",CARTHAGE_BUILD,dest,outpath.display());
//            fs::create_dir_all(output).unwrap();
//        } else {
//            println!(
//                "File {} extracted to \"{}\" ({} bytes)",
//                i,
//                outpath.display(),
//                file.size()
//            );
//            if let Some(p) = outpath.parent() {
//                if !p.exists() {
//                    let output = format!("{}/{}/{}",CARTHAGE_BUILD,dest,p.display());
//                    fs::create_dir_all(output).unwrap();
//                }
//            }
//            let output = format!("{}/{}/{}",CARTHAGE_BUILD,dest,outpath.display());
//            let mut outfile = fs::File::create(&output).unwrap();
//            std::io::copy(&mut file, &mut outfile).unwrap();
//        }
//
//        // Get and Set permissions
//        #[cfg(unix)]
//        {
//            use std::os::unix::fs::PermissionsExt;
//            if let Some(mode) = file.unix_mode() {
//                let output = format!("{}/{}/{}",CARTHAGE_BUILD,dest,outpath.display());
//                fs::set_permissions(output, fs::Permissions::from_mode(mode)).unwrap();
//            }
//        }
//    }
//}


//async fn download_from_s3(filename: &str,prefix: &str, bucket: String) -> Result<(),Box<dyn std::error::Error>>{
//    println!("downloading file right now");
//    let s3_client = S3Client::new(Region::UsWest1);
//    let pathStr = filename.to_string().split("/").last().unwrap().to_string();
//    let key = format!("{}/{}",prefix,pathStr).to_string();
//    println!("{}, {}",bucket, key);
//    let get_req = GetObjectRequest {
//        bucket: bucket,
//        key: key,
//       // key: "output/Alamofire.xcframework.zip".to_string(),
//        ..Default::default()
//    };
//    //let mut result; //s3_client.get_object(get_req).await.expect("error");
//    let mut result = s3_client.get_object(get_req).await;
//
//    let stream = result?.body.take().expect("no body");
//    println!("fetched {}", pathStr.clone());
//    let mut body = stream.into_async_read();
//    let mut file = tokio::fs::File::create(filename).await.unwrap();
//    tokio::io::copy(&mut body,&mut file).await;
//    
//
//    return Result::Ok(())
//
//
//}

//async fn upload_to_s3(filename: &str,prefix: &str, bucket:String) -> Result<(), Box<dyn std::error::Error>>{
//    let s3_client = S3Client::new(Region::UsWest1);
//    println!("uploading {}", filename);
//    let mut file = tokio::fs::File::open(filename).await?;
//    let mut buffer = Vec::new();    
//    file.read_to_end(&mut buffer).await?;
//    let pathStr = filename.to_string().split("/").last().unwrap().to_string();
//    let result = s3_client.put_object(PutObjectRequest {
//        bucket: bucket,
//        key: format!("{}/{}",prefix,pathStr).to_string(),
//        body: Some(StreamingBody::from(buffer)),
//        ..Default::default()
//    }).await?;
//    Ok(())
//}


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
    let mut cache_prefix = "output";
    if let Some(p) = matches.value_of("CachePrefix"){
        cache_prefix = p;
    }
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
                let result = cache::s3::download_from_s3(dest_dir.as_str(),cache_prefix,s3_bucket).await.unwrap_or_else(|e| {
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
        for frame in frameworks {
            println!("{}/{}",CARTHAGE_BUILD,frame);
            let src_dir = format!("{}/{}",CARTHAGE_BUILD,frame);
            let dest_dir = format!("{}/build/{}/{}.zip",expanded_str,cache_prefix,frame);
            println!("{}",dest_dir);
            let path = Path::new(dest_dir.as_str());
            if(path.exists()) {
                println!("framework {} already zipped", frame);
                cache::s3::upload_to_s3(dest_dir.as_str(),cache_prefix,pun.cache.s3_bucket.clone()).await;
            }else{
                let file = File::create(&path).unwrap();
                let walkdir = WalkDir::new(src_dir.to_string());
                let it = walkdir.into_iter();
                utils::archive::zip_dir(&mut it.filter_map(|e| e.ok()), src_dir.as_str(), file, zip::CompressionMethod::DEFLATE);
                cache::s3::upload_to_s3(dest_dir.as_str(),cache_prefix,pun.cache.s3_bucket.clone()).await;
            }

        }
    } 
}




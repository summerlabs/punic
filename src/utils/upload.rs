use crate::punfile::data::PunFile;
use crate::utils::scan::scan_xcframeworks;
use crate::{cache, utils};
use clap::{ArgMatches, Values};
use futures::future::join_all;
use std::borrow::Cow;
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;
use crate::punfile::data::Repository;
use std::collections::HashMap;
        


pub async fn upload_dependencies<'a>(
    punfile: PunFile,
    matches: &&ArgMatches<'a>,
    expanded_str: Cow<'a, str>,
) {
    let ignore_local_cache = matches.is_present(crate::IGNORE_LOCAL_CACHE);
    let mut children = vec![];
    let requested_frameworks: Vec<&str> = matches
        .values_of(crate::OVERRIDE_DEPENDENCIES_COMMAND)
        .unwrap_or(Values::default())
        .collect();
    let mut frameworks = scan_xcframeworks(punfile.configuration.output.clone());

    if !requested_frameworks.is_empty() {
        let mut filtered_frameworks: Vec<String> = Vec::new();
        for dep in &requested_frameworks {
            let temp = frameworks.iter().find(|item| item.contains(dep));
            if temp.is_none() {
                println!("{} is not a dependency", dep);
            } else {
                let frame = temp.unwrap();
                filtered_frameworks.push(frame.to_string());
            }
        }
        frameworks = filtered_frameworks;
    }
    for frame in frameworks {
        let output = punfile.configuration.output.clone();
        let cache_prefix = punfile.configuration.prefix.clone();
        let bucket_name = punfile.configuration.s3_bucket.clone();
        let default_repo = Repository {
            name: "".to_string(),
            repo_name: "".to_string(),
            version: "".to_string()
        };

        let expanded_frameworks: Vec<&str> = frame.split(".").collect();


        let framework_key = expanded_frameworks.get(0).unwrap();

        let framework = punfile.frameworks.iter().find(|item| item.name.contains(framework_key)).unwrap_or(&default_repo);

        if framework.version.is_empty() {
            println!("Found {}/{}", &output, frame);
        } else {
            println!("Found {}/{} @version {}", &output, frame, framework.version);
        }

        let src_dir = format!("{}/{}", output, frame);
        let dest_dir = { format!("{}/build/{}/{}.zip", expanded_str, cache_prefix, frame) };
        let prefix = cache_prefix.clone();
        let version = framework.version.clone();
        let _empty_string = String::from("");
        let prefix = match framework.version.as_str() {
                "" => cache_prefix.clone(),
                _ => format!("{}/{}",cache_prefix.clone(),version)
        };
        // If the cache does not exist or we're ignoring the cache -> zip the files
        if ignore_local_cache || !Path::new(&dest_dir).exists() {
            let dest = dest_dir.clone();
            let task = tokio::spawn(async move {
                let file = File::create(dest).unwrap();
                let walk_dir = WalkDir::new(src_dir.to_string());
                let it = walk_dir.into_iter();
                utils::archive::zip_dir(
                    &mut it.filter_map(|e| e.ok()),
                    src_dir.as_str(),
                    file,
                    zip::CompressionMethod::DEFLATE,
                )
                .ok();
                cache::s3::upload(dest_dir, prefix.to_string(), bucket_name)
                    .await
                    .ok();
            });
            children.push(task);
        } else {
            println!("Already zipped {}", frame);
            let dest = dest_dir;
            let pref = prefix.to_string();
            let task = tokio::spawn(async move {
                cache::s3::upload(dest, pref, bucket_name).await.ok();
            });
            children.push(task);
        }
    }
    join_all(children).await;
}

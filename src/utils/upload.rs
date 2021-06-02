use crate::punfile::data::PunFile;
use crate::utils::scan::scan_xcframeworks;
use crate::{cache, utils};
use clap::{ArgMatches, Values};
use futures::future::join_all;
use std::borrow::Cow;
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;

pub async fn upload_dependencies<'a>(
    punfile: PunFile,
    matches: &&ArgMatches<'a>,
    expanded_str: Cow<'a, str>,
) {
    let force_command = matches.value_of("ForceCommand").unwrap_or("false");
    let cache_prefix = punfile.configuration.prefix;
    let mut children = vec![];
    let requested_frameworks: Vec<&str> = matches
        .values_of("dependencies")
        .unwrap_or(Values::default())
        .collect();
    let mut frameworks = scan_xcframeworks();
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
        println!("Found {}/{}", crate::CARTHAGE_BUILD, frame);
        let src_dir = format!("{}/{}", crate::CARTHAGE_BUILD, frame);
        let dest_dir = { format!("{}/build/{}/{}.zip", expanded_str, cache_prefix, frame) };
        let bucket_name = punfile.configuration.s3_bucket.clone();
        let prefix = cache_prefix.clone();
        if Path::new(&dest_dir).exists() && !force_command.eq("true") {
            println!("Already zipped {}", frame);
            let dest = dest_dir;
            let pref = prefix.to_string();
            let task = tokio::spawn(async move {
                cache::s3::upload(dest, pref, bucket_name).await.ok();
            });
            children.push(task);
        } else {
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
        }
    }
    join_all(children).await;
}

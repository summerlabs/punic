use crate::punfile::data::{PunFile, Repository};
use crate::{cache, utils};
use clap::{ArgMatches, Values};
use futures::future::join_all;
use std::borrow::Cow;
use std::path::Path;

pub async fn download_dependencies<'a>(
    punfile: PunFile,
    matches: &&ArgMatches<'a>,
    cache_dir: Cow<'a, str>,
) {
    let ignore_local_cache = matches.is_present(crate::IGNORE_LOCAL_CACHE);
    let ignore_output_cache = matches.is_present(crate::IGNORE_OUTPUT_CACHE);
    let mut children = vec![];
    let requested_frameworks: Vec<Repository> = matches
        .values_of(crate::OVERRIDE_DEPENDENCIES_COMMAND)
        .unwrap_or(Values::default())
        .map(|it| Repository {
            repo_name: String::from(it),
            name: String::from(it),
            version: String::from(it)
        })
        .collect();
    let mut frameworks = punfile.frameworks;
    if !requested_frameworks.is_empty() {
        let mut filtered_frameworks: Vec<Repository> = Vec::new();
        for dep in &requested_frameworks {
            let temp = frameworks
                .iter()
                .find(|item| item.repo_name.eq(&dep.repo_name));
            if temp.is_none() {
                println!("{} is not a dependency in the Punfile.", dep.repo_name);
            } else {
                let frame = temp.unwrap();
                filtered_frameworks.push(Repository {
                    repo_name: frame.repo_name.to_string(),
                    name: frame.name.to_string(),
                    version: frame.version.to_string()
                });
            }
        }
        frameworks = filtered_frameworks;
    }

    for dependencies in frameworks {
        let output = punfile.configuration.output.clone();
        let cache_prefix = punfile.configuration.prefix.clone();
        let framework_name = format!("{}.xcframework", dependencies.name);
        let version = dependencies.version.clone();
        let xcf_cache_dir = format!(
            "{}/build/{}/{}.xcframework.zip",
            cache_dir, cache_prefix, dependencies.name
        )
        .to_string();
        let xcf_cache_path = Path::new(xcf_cache_dir.as_str());
        // If the framework does not exist or we're ignoring the local cache -> download
        if !xcf_cache_path.exists() || ignore_local_cache {
            if !xcf_cache_path.exists() {
                println!("Not found {}", xcf_cache_dir);
            } else if ignore_local_cache {
                println!("Ignoring {}", xcf_cache_dir);
            }
            let s3_bucket = punfile.configuration.s3_bucket.clone();
            let empty = String::from("");
            let prefix = match dependencies.version {
                empty => format!("{}/{}", version.clone(), cache_prefix.clone()),
                _ => cache_prefix.clone(),
            };

            let task = tokio::spawn(async move {
                cache::s3::download_from_s3(
                    xcf_cache_dir.to_string(),
                    prefix.to_string(),
                    s3_bucket,
                )
                .await
                .ok();
                let path = Path::new(xcf_cache_dir.as_str());
                if path.exists() {
                    utils::archive::extract_zip(
                        &output,
                        xcf_cache_dir.as_str(),
                        framework_name.as_str(),
                    );
                }
            });
            children.push(task);
        } else {
            let xfr_output_dir = format!("{}/{}.xcframework", output, dependencies.name);
            let xcf_output_path = Path::new(xfr_output_dir.as_str());
            // If the output path does not exist or we're ignoring it -> copy files over
            if !xcf_output_path.exists() || ignore_output_cache {
                if !xcf_output_path.exists() {
                    println!("Not found {}", xfr_output_dir);
                } else if ignore_output_cache {
                    println!("Ignoring {}", xfr_output_dir);
                }
                let task = tokio::spawn(async move {
                    utils::archive::extract_zip(
                        &output,
                        xcf_cache_dir.as_str(),
                        framework_name.as_str(),
                    );
                });
                children.push(task);
            } else {
                println!("Already downloaded {}", xcf_cache_path.display());
            }
        }
    }
    join_all(children).await;
}

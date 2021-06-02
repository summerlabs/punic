use crate::punfile::data::{PunFile, Repository};
use crate::{cache, utils};
use clap::{ArgMatches, Values};
use futures::future::join_all;
use std::borrow::Cow;
use std::path::Path;

pub async fn download_dependencies<'a>(
    punfile: PunFile,
    matches: &&ArgMatches<'a>,
    expanded_str: Cow<'a, str>,
) {
    let force_command = matches.value_of("ForceCommand").unwrap_or("false");
    let mut children = vec![];
    let requested_frameworks: Vec<Repository> = matches
        .values_of("dependencies")
        .unwrap_or(Values::default())
        .map(|it| Repository {
            repo_name: String::from(it),
            name: String::from(it),
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
                println!("{} is not a dependency", dep.repo_name);
            } else {
                let frame = temp.unwrap();
                filtered_frameworks.push(Repository {
                    repo_name: frame.repo_name.to_string(),
                    name: frame.name.to_string(),
                });
            }
        }
        frameworks = filtered_frameworks;
    }

    for dependencies in frameworks {
        let output = punfile.configuration.output.clone();
        let cache_prefix = punfile.configuration.prefix.clone();
        let framework_name = format!("{}.xcframework", dependencies.name);
        let dest_dir = format!(
            "{}/build/{}/{}.xcframework.zip",
            expanded_str, cache_prefix, dependencies.name
        )
        .to_string();
        let path = Path::new(dest_dir.as_str());
        let prefix = cache_prefix.clone();
        if path.exists() && !force_command.eq("true") {
            let dep_path_format = format!("{}/{}.xcframework", output, dependencies.name);
            let dep_path = Path::new(dep_path_format.as_str());
            println!("{}", dep_path_format);
            if !dep_path.exists() {
                let task = tokio::spawn(async move {
                    utils::archive::extract_zip(
                        &output,
                        dest_dir.as_str(),
                        framework_name.as_str(),
                    );
                });
                children.push(task);
            } else {
                println!("Already downloaded {}", path.display());
            }
        } else {
            let s3_bucket = punfile.configuration.s3_bucket.clone();
            let task = tokio::spawn(async move {
                cache::s3::download_from_s3(dest_dir.to_string(), prefix.to_string(), s3_bucket)
                    .await
                    .ok();
                let path = Path::new(dest_dir.as_str());
                if path.exists() {
                    utils::archive::extract_zip(
                        &output,
                        dest_dir.as_str(),
                        framework_name.as_str(),
                    );
                }
            });
            children.push(task);
        }
    }
    join_all(children).await;
}

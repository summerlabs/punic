


use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3, S3Client, StreamingBody, GetObjectOutput, GetObjectError, PutObjectError, PutObjectOutput};
use rusoto_core::{Region,ByteStream,RusotoError,RusotoResult};

use std::future::Future;
use tokio::io::{self,AsyncReadExt, AsyncWrite,AsyncWriteExt};
use futures::{TryFutureExt, StreamExt, TryStreamExt};
use std::fs::File;
use tokio_util::codec;
use rusoto_core::signature::SignedRequestPayload::Stream;
use std::ptr::null;

pub async fn download_from_s3(filename: String,prefix: String, bucket: String) -> Result<(),Box<dyn std::error::Error>>{
    let s3_client = S3Client::new(Region::UsWest1);

    let f_name = { filename.clone() };
    let path_str = f_name.split("/").last().unwrap_or("");
    let object_key = format!("{}/{}",prefix,path_str).to_string();
    println!("Downloading {}/{}...", bucket, object_key);

    let request = GetObjectRequest {
        bucket,
        key: object_key,
        ..Default::default()
    };

    let stream = s3_client.get_object(request).await;

    let stream = stream?.body.take().expect("no body");
    let mut body = stream.into_async_read();
    let mut file = tokio::fs::File::create(filename).await.unwrap();
    tokio::io::copy(&mut body,&mut file).await;

    Ok(())
}

pub async fn upload(filename: String,prefix:String, bucket:String) -> Result<(),Box<dyn std::error::Error>>{
    let s3_client = S3Client::new(Region::UsWest1);
    let mut buffer = Vec::new();
    let f_name = { filename.clone() };
    let path_str = f_name.split("/").last().unwrap().to_string();
    let file = File::open(f_name).unwrap();
    let mut tokio_file = tokio::fs::File::from_std(file);
    tokio_file.read_to_end(&mut buffer).await;
    let object_key = format!("{}/{}", prefix, path_str).to_string();
    println!("Uploading {}/{}...", bucket, object_key);
    s3_client.put_object(PutObjectRequest {
        bucket,
        key: object_key,
        body: Some(StreamingBody::from(buffer)),
        ..Default::default()
    }).await;

    Ok(())
}




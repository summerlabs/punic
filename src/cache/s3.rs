


use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3, S3Client, StreamingBody, GetObjectOutput, GetObjectError, PutObjectError, PutObjectOutput};
use rusoto_core::{Region,ByteStream,RusotoError,RusotoResult};

use std::future::Future;
use tokio::io::{self,AsyncReadExt, AsyncWrite,AsyncWriteExt};
use futures::{TryFutureExt, StreamExt, TryStreamExt};
use std::fs::File;
use tokio_util::codec;
use rusoto_core::signature::SignedRequestPayload::Stream;
use std::ptr::null;

pub async fn download_from_s3(filename: &str,prefix: &str, bucket: String) -> Result<(),Box<dyn std::error::Error>>{
    println!("downloading {}{}", prefix, filename);
    let s3_client = S3Client::new(Region::UsWest1);
    let pathStr = filename.to_string().split("/").last().unwrap().to_string();
    let key = format!("{}/{}",prefix,pathStr).to_string();
    println!("{}, {}",bucket, key);
    let get_req = GetObjectRequest {
        bucket: bucket,
        key: key,
        ..Default::default()
    };

    let mut result = s3_client.get_object(get_req).await;

    let stream = result?.body.take().expect("no body");
    println!("Downloaded {}", pathStr.clone());
    let mut body = stream.into_async_read();
    let mut file = tokio::fs::File::create(filename).await.unwrap();
    tokio::io::copy(&mut body,&mut file).await;
    
    return Result::Ok(())
}

pub async fn upload(filename: String,prefix:String, bucket:String) -> Result<(),Box<dyn std::error::Error>>{
    let s3_client = S3Client::new(Region::UsWest1);
    let mut buffer = Vec::new();
    let fname = {
        filename.clone()
    };
    let pathStr = fname.split("/").last().unwrap().to_string();
    let file = File::open(fname).unwrap();
    let mut tokio_file = tokio::fs::File::from_std(file);
    tokio_file.read_to_end(&mut buffer).await;
    //.unwrap_or_else(|e| panic!("unable to read byte stream"));
    // let byte_stream = codec::FramedRead::new(tokio_file, codec::BytesCodec::new()).map(|r| r.unwrap().freeze());
    println!("Uploading {}...", pathStr);
    s3_client.put_object(PutObjectRequest {
        bucket: bucket,
        key: format!("{}/{}",prefix,pathStr).to_string(),
        body: Some(StreamingBody::from(buffer)),
        ..Default::default()
    }).await;

    Ok(())
}




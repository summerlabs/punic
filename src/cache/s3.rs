


use rusoto_s3::{GetObjectRequest,PutObjectRequest, S3, S3Client, StreamingBody,GetObjectOutput,GetObjectError};
use rusoto_core::{Region,ByteStream,RusotoError,RusotoResult};
use tokio::io::{self,AsyncReadExt, AsyncWrite,AsyncWriteExt};



pub async fn download_from_s3(filename: &str,prefix: &str, bucket: String) -> Result<(),Box<dyn std::error::Error>>{
    println!("downloading file right now");
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
    println!("fetched {}", pathStr.clone());
    let mut body = stream.into_async_read();
    let mut file = tokio::fs::File::create(filename).await.unwrap();
    tokio::io::copy(&mut body,&mut file).await;
    

    return Result::Ok(())


}

pub async fn upload_to_s3(filename: &str,prefix: &str, bucket:String) -> Result<(), Box<dyn std::error::Error>>{
    let s3_client = S3Client::new(Region::UsWest1);
    println!("uploading {}", filename);
    let mut file = tokio::fs::File::open(filename).await?;
    let mut buffer = Vec::new();    
    file.read_to_end(&mut buffer).await?;
    let pathStr = filename.to_string().split("/").last().unwrap().to_string();
    let result = s3_client.put_object(PutObjectRequest {
        bucket: bucket,
        key: format!("{}/{}",prefix,pathStr).to_string(),
        body: Some(StreamingBody::from(buffer)),
        ..Default::default()
    }).await?;
    Ok(())
}

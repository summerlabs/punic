use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, StreamingBody, S3};
use std::fs::File;
use tokio::io::AsyncReadExt;

pub async fn download_from_s3(
    filename: String,
    prefix: String,
    bucket: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let s3_client = S3Client::new(Region::default());

    let f_name = { filename.clone() };
    let path_str = f_name.split("/").last().unwrap_or("");
    let object_key = format!("{}/{}", prefix, path_str).to_string();
    let _bucket = bucket.clone();
    let _object_key = object_key.clone();

    println!("Downloading {}/{}...", bucket.clone(), object_key);

    let request = GetObjectRequest {
        bucket,
        key: object_key,
        ..Default::default()
    };

    let stream = s3_client.get_object(request).await;

    let mut output = match stream {
        Ok(output) => output,
        Err(error) => panic!(error.to_string()),
    };

    let stream = output.body.take().expect("No Content");

    let mut body = stream.into_async_read();
    let mut file = tokio::fs::File::create(filename).await.unwrap();
    tokio::io::copy(&mut body, &mut file).await.ok();

    println!("Downloaded {}/{}", _bucket, _object_key);

    Ok(())
}

pub async fn upload(
    filename: String,
    prefix: String,
    bucket: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let s3_client = S3Client::new(Region::default());
    let mut buffer = Vec::new();
    let f_name = { filename.clone() };
    let path_str = f_name.split("/").last().unwrap().to_string();
    let file = File::open(f_name).unwrap();
    let mut tokio_file = tokio::fs::File::from_std(file);
    tokio_file.read_to_end(&mut buffer).await.ok();
    let object_key = format!("{}/{}", prefix, path_str).to_string();
    let object_key2 = object_key.clone();
    let bucket2 = bucket.clone();
    println!("Uploading {}/{}...", bucket, object_key);
    s3_client
        .put_object(PutObjectRequest {
            bucket,
            key: object_key,
            body: Some(StreamingBody::from(buffer)),
            ..Default::default()
        })
        .await
        .ok();
    println!("Uploaded {}/{}", bucket2, object_key2);

    Ok(())
}

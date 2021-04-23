# Punic

![elephant](assets/elephant_2.png) 

**Punic** is a remote caching CLI built for [Carthage](https://github.com/Carthage/Carthage)
that exclusively supports Apple's `.xcframeworks`.

**Features**
- ✅ Easy remote caching via [AWS S3](https://aws.amazon.com/s3/)
- ✅ Easy CI/CD integration
- ✅ Support for versioned dependencies

Please give us a `star` if you find this CLI useful!


# Installation

```bash
brew tap summerlabs/homebrew-punic
brew install punic
```

**AWS Credentials**

Make sure you have your AWS config and credentials [setup](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html).

They are keys that **Punic** uses to upload your frameworks into AWS
and will be automatically setup for you when you run `aws configure`.

Here is an example after you run the setup successfully.

`~/.aws/config`
```bash
[default]
region = us-west-1
output = json
```
`~/.aws/credentials`
```bash
[default]
aws_access_key_id = {SOME_ACCESS_KEY_ID}
aws_secret_access_key = {SOME_SECRET_KEY}
```



# Documentation

### Punfile

**Punic** looks for a `Punfile` to determine which dependencies to download 
as well as configuring the path of your local cache.


**Example Punfile**

```yaml
# Configure Punic
configuration:
  # save dependencies into this directory
  # ie. //some-remote-bucket/1.0.1/Alamofire/Alamofire.xcframework
  # 
  prefix: 1.0.1
  # local cache location
  local: ~/Library/Caches/Punic
  # aws bucket location
  s3Bucket: some-remote-bucket
  
# Search your Carthage/Build directory for these .xcframeworks
dependencies:
# single framework definition
- AlamoFire:
  - name: AlamoFire
# multiple frameworks definition sometimes created by one library
- CocoaLumberjack:
  - name: CocoaLumberjack
  - name: CocoaLumberjackSwift
  - name: CocoaAsyncSocket
```

## Commands

After building your `.xcframeworks` using 
```bash
carthage update --use-xcframeworks
```
They will be located in the top level `Carthage/Build` directory.

**Download .xcframeworks**
```bash
punic download
```
**Upload .xcframeworks**
```bash
punic upload
```

## Carthage-less Support

**Punic** was built with `Carthage` in mind as it essentially builds frameworks
for consumption. However, **Punic** by itself is simply a remote caching tool that simply 
uploads and downloads a directory of frameworks in isolation. Presumably, the download path
can point to anything. Right now it points to the project's `Carthage/Build` folder
but later on will be configurable to point to any folder. Right now, it works very
well with `Carthage` and should be used as such.

## Developer Support

**Punic** is a new framework that was made to help our team
achieve remote caching with Apple's latest `.xcframework` change.
If you find any issues using the CLI, don't hesitate to open one up 
to help us bug splash.

For all you `Rust` developers, feel free to contribute to this framework
and help us grow the CLI.





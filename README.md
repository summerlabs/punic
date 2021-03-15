# PUNIC


# GETTING STARTED
---
Caching framework for ios xcframeworks dependency

# INSTALLATION
---
Currently punic can be installed as a binary or you can use homebrew I recommend using homebrew to run the installation
```bash
    brew tap summerlabs/homebrew-punic
    brew install punic
```

# USAGE
---
Punic looks for a Punfile to determine which dependencies to download and also configure where your s3 cache and local cache should live
its very similiar to rome below is an example of what it should look like
```yaml
cache:
    local: ~/Library/Caches/Punic
    s3Bucket: my-bucket
repositoryMap:
- AlamoFire:
    - name: AlamoFire
```
to download from cache
```bash
    pun download to download assets 
```
to upload your s3 cache
```bash
    pun upload to upload assets 
```




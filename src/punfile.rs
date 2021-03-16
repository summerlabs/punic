


use serde::{Serialize, Deserialize};


pub mod data {

    pub struct CacheSetting {
        pub prefix: String,
        pub local: String,
        pub s3_bucket: String
    }

    pub struct PunFile {
        pub cache: CacheSetting,
        pub frameworks: Vec<Repository>
    }


    pub struct Repository {
        pub repo_name: String,
        pub name: String,
        //pub version: String,
        pub platforms: Vec<String>
    }

}




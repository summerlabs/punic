


use serde::{Serialize, Deserialize};


pub mod data {

    pub struct Configuration {
        pub prefix: String,
        pub local: String,
        pub s3_bucket: String
    }

    pub struct PunFile {
        pub configuration: Configuration,
        pub frameworks: Vec<Repository>
    }


    pub struct Repository {
        pub repo_name: String,
        pub name: String
    }

}




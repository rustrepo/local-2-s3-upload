use std::env;

#[derive(Clone)]
pub struct Config {
    pub directory_path: String,
    pub bucket_name: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

impl Config {
    pub fn load() -> Self {
        Config {
            directory_path: env::var("DIRECTORY_PATH").unwrap_or_else(|_| "./files".to_string()),
            bucket_name: env::var("BUCKET_NAME").unwrap_or_else(|_| "bucket-name".to_string()),
            region: env::var("AWS_REGION").unwrap_or_else(|_| "region".to_string()),
            access_key: env::var("AWS_ACCESS_KEY").unwrap_or_else(|_| "abc".to_string()),
            secret_key: env::var("AWS_SECRET_KEY").unwrap_or_else(|_| "xyz".to_string()),
        }
    }
}

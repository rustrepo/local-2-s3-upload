mod uploader;
mod signer;
mod file_utils;
mod config;

use uploader::upload_directory;
use config::Config;
use sysinfo::{System, SystemExt};

fn main() {
    dotenv::dotenv().ok();

    let config = Config::load();
    
    let system = System::new_all();
    let initial_memory = system.used_memory();

    let results = upload_directory(
        &config.directory_path,
        &config.bucket_name,
        &config.region,
        &config.access_key,
        &config.secret_key,
    );

    let final_memory = system.used_memory();
    let successful_uploads = results.iter().filter(|(_, status)| !status.contains("Failed")).count();

    println!("Memory used: {} KB", final_memory - initial_memory);
    println!("Successfully uploaded: {}", successful_uploads);
}

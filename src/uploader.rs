use crate::signer::{get_signature_key, sign_string};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use walkdir::WalkDir;
use urlencoding::encode;
use sha2::{Digest, Sha256};

pub fn upload_file_to_s3(
    file_path: &str,
    bucket_name: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
    client: &Client,
    signing_key: &[u8],
    date: &str,
    date_short: &str,
) -> Result<String, String> {
    let file_name = Path::new(file_path)
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?
        .to_string_lossy()
        .to_string();

    if file_name == ".DS_Store" {
        return Ok("File ignored: .DS_Store".to_string());
    }

    let encoded_file_name = encode(&file_name);
    let new_file_name = format!("{}-{}", date, encoded_file_name);

    let mut file = BufReader::new(File::open(file_path).map_err(|e| e.to_string())?);
    let mut content = Vec::new();
    file.read_to_end(&mut content).map_err(|e| e.to_string())?;

    let host = format!("{}.s3.{}.amazonaws.com", bucket_name, region);
    let endpoint = format!("https://{}/{}", host, new_file_name);

    let canonical_headers = format!("host:{}\nx-amz-date:{}\n", host, date);
    let signed_headers = "host;x-amz-date";
    let payload_hash = hex::encode(Sha256::digest(&content));
    let canonical_request = format!(
        "PUT\n/{}\n\n{}\n{}\n{}",
        new_file_name, canonical_headers, signed_headers, payload_hash
    );

    let algorithm = "AWS4-HMAC-SHA256";
    let credential_scope = format!("{}/{}/{}/aws4_request", date_short, region, "s3");
    let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));
    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        algorithm, date, credential_scope, canonical_request_hash
    );

    let signature = sign_string(signing_key, &string_to_sign);
    let authorization_header = format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm, access_key, credential_scope, signed_headers, signature
    );

    let mut headers = HeaderMap::new();
    headers.insert("x-amz-date", HeaderValue::from_str(date).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    headers.insert("Authorization", HeaderValue::from_str(&authorization_header).unwrap());
    headers.insert("x-amz-content-sha256", HeaderValue::from_str(&payload_hash).unwrap());

    let response = client
        .put(&endpoint)
        .headers(headers)
        .body(content)
        .send()
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok(format!("File uploaded successfully: {}", new_file_name))
    } else {
        Err(format!(
            "Failed to upload file: {}. Status: {}",
            new_file_name,
            response.status()
        ))
    }
}

pub fn upload_directory(
    dir_path: &str,
    bucket_name: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
) -> Vec<(String, String)> {
    let client = Client::new();
    let date = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let date_short = &date[..8];
    let signing_key = get_signature_key(secret_key, date_short, region, "s3");

    WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                let file_path = path.to_string_lossy().to_string();
                let result = upload_file_to_s3(
                    &file_path, bucket_name, region, access_key, secret_key, &client, &signing_key, &date, date_short,
                );
                Some((file_path, result.unwrap_or_else(|e| e)))
            } else {
                None
            }
        })
        .collect()
}

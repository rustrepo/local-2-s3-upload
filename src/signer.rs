use hmac::{Hmac, Mac};
use sha2::{Sha256};

type HmacSha256 = Hmac<Sha256>;

pub fn get_signature_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let mut key = HmacSha256::new_from_slice(format!("AWS4{}", secret).as_bytes()).unwrap();
    key.update(date.as_bytes());
    let mut key = HmacSha256::new_from_slice(&key.finalize().into_bytes()).unwrap();
    key.update(region.as_bytes());
    let mut key = HmacSha256::new_from_slice(&key.finalize().into_bytes()).unwrap();
    key.update(service.as_bytes());
    let mut key = HmacSha256::new_from_slice(&key.finalize().into_bytes()).unwrap();
    key.update(b"aws4_request");
    key.finalize().into_bytes().to_vec()
}

pub fn sign_string(key: &[u8], string_to_sign: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(string_to_sign.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

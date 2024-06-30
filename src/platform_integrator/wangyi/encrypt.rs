#![allow(unused)]
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit};
use anyhow::Error;
use base64::prelude::BASE64_STANDARD;
use base64::Engine as _;
use lazy_static::lazy_static;
use md5::digest::core_api::CoreWrapper;
use md5::Digest;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use rsa::hazmat::rsa_encrypt;
use rsa::{pkcs8::DecodePublicKey, BigUint, RsaPublicKey};
use std::collections::HashMap;
use std::fmt::Debug;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;
type Aes128EcbDec = ecb::Decryptor<aes::Aes128>;

lazy_static! {
    static ref IV: [u8; 16] = *b"0102030405060708"; // 初始化向量
    static ref PRESET_KEY: [u8; 16] = *b"0CoJUm6Qyw8W8jud"; // 预设AES密钥
    static ref LINUXAPI_KEY: [u8; 16] = *b"rFgB&h#%2?^eDg:Q"; // Linux API密钥
    static ref EAPI_KEY: [u8; 16] = *b"e82ckenh8dichen8"; // eAPI密钥
    static ref RSA_PUBKEY: RsaPublicKey = {
        let pub_key_pem = include_str!("rsa_pub_key.pem");
        RsaPublicKey::from_public_key_pem(pub_key_pem).unwrap()
    };
}

#[derive(Debug)]
pub enum AesMode {
    CBC,
    ECB,
}

pub fn aes_encrypt(buffer: &[u8], key: &[u8; 16], iv: Option<&[u8; 16]>, mode: AesMode) -> Vec<u8> {
    match mode {
        AesMode::CBC => {
            let iv = iv.expect("IV must be provided for CBC mode");
            let mut buf = vec![0u8; buffer.len() + 16]; // buffer size with padding
            buf[..buffer.len()].copy_from_slice(buffer);
            let ct = Aes128CbcEnc::new(key.into(), iv.into())
                .encrypt_padded_mut::<Pkcs7>(&mut buf, buffer.len())
                .unwrap();
            ct.to_vec()
        }
        AesMode::ECB => {
            let mut buf = vec![0u8; buffer.len() + 16]; // buffer size with padding
            buf[..buffer.len()].copy_from_slice(buffer);
            let ct = Aes128EcbEnc::new(key.into())
                .encrypt_padded_mut::<Pkcs7>(&mut buf, buffer.len())
                .unwrap();
            ct.to_vec()
        }
    }
}

pub fn aes_decrypt(
    ciphertext: &[u8],
    key: &[u8; 16],
    iv: Option<&[u8; 16]>,
    mode: AesMode,
) -> Vec<u8> {
    match mode {
        AesMode::CBC => {
            let iv = iv.expect("IV must be provided for CBC mode");
            let mut buf = ciphertext.to_vec();
            let pt = Aes128CbcDec::new(key.into(), iv.into())
                .decrypt_padded_mut::<Pkcs7>(&mut buf)
                .unwrap();
            pt.to_vec()
        }
        AesMode::ECB => {
            let mut buf = ciphertext.to_vec();
            let pt = Aes128EcbDec::new(key.into())
                .decrypt_padded_mut::<Pkcs7>(&mut buf)
                .unwrap();
            pt.to_vec()
        }
    }
}

fn rsa_encrypt_to_hex(message: &[u8]) -> Result<String, rsa::errors::Error> {
    let prefix = vec![0u8; 128 - message.len()];
    let message = [&prefix[..], message].concat();
    let encrypted =
        rsa_encrypt(&RSA_PUBKEY.to_owned(), &BigUint::from_bytes_be(&message))?.to_bytes_be();
    let encrypted_str: String = hex::encode(encrypted);
    Ok(encrypted_str)
}

pub fn weapi(data: &str) -> Result<HashMap<&'static str, String>, Error> {
    let rng = thread_rng();

    let random_key: [u8; 16] = rng
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(|byte| byte as u8)
        .collect::<Vec<u8>>()
        .try_into()
        .expect("Failed to generate a random key");
    // let random_key = b"0CoJUm6Qyw8W8jud";
    let encrypted_once = aes_encrypt(data.as_bytes(), &PRESET_KEY, Some(&IV), AesMode::CBC);
    let encrypted_once_base64 = BASE64_STANDARD.encode(&encrypted_once);

    let encrypted_twice = aes_encrypt(
        encrypted_once_base64.as_bytes(),
        &random_key,
        Some(&IV),
        AesMode::CBC,
    );

    let encrypted_twice_base64 = BASE64_STANDARD.encode(&encrypted_twice);

    let mut reversed_key = random_key.to_vec();
    reversed_key.reverse();
    let enc_sec_key = rsa_encrypt_to_hex(&reversed_key)?;
    let mut result = HashMap::new();
    result.insert("params", encrypted_twice_base64);
    result.insert("encSecKey", enc_sec_key);
    Ok(result)
}

pub(crate) fn eapi(url: &str, data: &str) -> HashMap<&'static str, std::string::String> {
    let message = format!("nobody{}use{}md5forencrypt", url, data);

    let mut hasher: CoreWrapper<md5::Md5Core> = Digest::new();
    hasher.update(message.as_bytes());
    let digest = hex::encode(hasher.finalize());

    let data = format!("{}-36cd479b6b5-{}-36cd479b6b5-{}", url, data, digest);

    let encrypted_data = aes_encrypt(data.as_bytes(), &EAPI_KEY, None, AesMode::ECB);
    let mut result = HashMap::new();
    result.insert("params", hex::encode_upper(encrypted_data));
    result
}

pub fn linux_api(text: &str) -> HashMap<&'static str, String> {
    let encrypted = aes_encrypt(text.as_bytes(), &LINUXAPI_KEY, None, AesMode::ECB);
    let digest = hex::encode_upper(&encrypted);
    let mut result = HashMap::new();
    result.insert("eparams", digest);
    result
}

#[test]
fn test() {
    // 要加密的明文
    let plaintext = b"hello world! this is my plaintext.";

    // CBC模式加密
    let ciphertext_cbc = aes_encrypt(plaintext, &PRESET_KEY, Some(&IV), AesMode::CBC);
    println!("CBC Ciphertext (hex): {:?}", hex::encode(&ciphertext_cbc));

    // CBC模式解密
    let decrypted_text_cbc = aes_decrypt(&ciphertext_cbc, &PRESET_KEY, Some(&IV), AesMode::CBC);
    println!(
        "CBC Decrypted text: {:?}",
        String::from_utf8(decrypted_text_cbc).unwrap()
    );

    // ECB模式加密
    let ciphertext_ecb = aes_encrypt(plaintext, &PRESET_KEY, None, AesMode::ECB);
    println!("ECB Ciphertext (hex): {:?}", hex::encode(&ciphertext_ecb));

    // ECB模式解密
    let decrypted_text_ecb = aes_decrypt(&ciphertext_ecb, &PRESET_KEY, None, AesMode::ECB);
    println!(
        "ECB Decrypted text: {:?}",
        String::from_utf8(decrypted_text_ecb).unwrap()
    );

    // RSA加密
    let encrypted_hex = rsa_encrypt_to_hex(plaintext).unwrap();
    println!("RSA Encrypted (hex): {:?}", encrypted_hex);

    // weapi加密
    let weapi_params = weapi("hello world!").unwrap();
    println!("weapi params: {:?}", weapi_params);

    // eapi加密
    let eapi_params = eapi("/api/v1/resource", "hello world!");
    println!("eapi params: {:?}", eapi_params);

    // linuxapi加密
    let linuxapi_params = linux_api("hello world!");
    println!("linuxapi params: {:?}", linuxapi_params);
}

#[test]
fn test_eapi() {
    let eapi_params = eapi(
        "/api/cloudsearch/pc",
        r#"{"s":"张惠妹","type":1,"limit":30,"total":true,"offset":0}"#,
    );
    println!("eapi params: {:?}", eapi_params);
}

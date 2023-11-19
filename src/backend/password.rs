use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use std::{
    fs::*,
    io::{Read, Write},
    path::Path,
};

pub fn generate_salt(username: &str) -> SaltString {
    let salt = SaltString::generate(&mut OsRng);

    // Stores the salts in a file
    File::options()
        .append(true) // Append, don't override
        .create(true) // Create the file if it does not exist
        .open(Path::new("salts.csv"))
        .and_then(|mut x| x.write_all(format!("{},{}", username, salt).as_bytes()))
        .unwrap();

    salt
}

pub fn fetch_salt(username: &str) -> SaltString {
    let mut file = File::open(Path::new("salts.csv")).unwrap();
    let mut buf = Vec::new();

    // Never do this in production, use a buffered reader and read line by line until you get what you need, also use async
    file.read_to_end(&mut buf).unwrap();

    let binding = String::from_utf8_lossy(&buf); // converts the Vec<u8> to a String, without checking for invalid utf8
    let salt = binding
        .lines()
        .find(|x| x.contains(username))
        .unwrap_or_default()
        .split(",")
        .nth(1)
        .unwrap_or_default(); // get the salt (nth 1) from the line that contains the username (nth 0)

    SaltString::from_b64(salt).unwrap()
}

pub fn hash(password: &str, salt: SaltString) -> String {
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap();
    password_hash.to_string()
}

pub fn verify(password_hash: SaltString, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(password_hash.as_str()).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

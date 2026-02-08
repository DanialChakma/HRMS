use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &str) -> String {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn verify_password(password: &str, hashed: &str) -> Result<(), argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let parsed = PasswordHash::new(hashed).unwrap();

    argon2.verify_password(password.as_bytes(), &parsed)
}

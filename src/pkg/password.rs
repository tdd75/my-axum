use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// Configuration for password hashing
#[derive(Debug, Clone)]
pub struct PasswordConfig {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            memory_cost: 4096,
            time_cost: 3,
            parallelism: 1,
        }
    }
}

/// Hash a password using Argon2
pub async fn hash_password(password: &str) -> anyhow::Result<String> {
    hash_password_with_config(password, &PasswordConfig::default()).await
}

/// Hash a password with custom configuration
pub async fn hash_password_with_config(
    password: &str,
    config: &PasswordConfig,
) -> anyhow::Result<String> {
    let password_bytes = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(
            config.memory_cost,
            config.time_cost,
            config.parallelism,
            None,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create Argon2 params: {}", e))?,
    );

    let password_hash = argon2
        .hash_password(password_bytes, &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against a hash
pub async fn verify_password(password: &str, hash: &str) -> anyhow::Result<()> {
    let password_bytes = password.as_bytes();
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

    let argon2 = Argon2::default();

    argon2
        .verify_password(password_bytes, &parsed_hash)
        .map_err(|e| anyhow::anyhow!("Password verification failed: {}", e))?;

    Ok(())
}

/// Hash a plain string password
pub async fn hash_password_string(password: &str) -> anyhow::Result<String> {
    hash_password(password).await
}

/// Generate a random salt string
pub fn generate_salt() -> String {
    SaltString::generate(&mut OsRng).to_string()
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> anyhow::Result<()> {
    if password.len() < 8 {
        return Err(anyhow::anyhow!(
            "Password must be at least 8 characters long"
        ));
    }

    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_upper {
        return Err(anyhow::anyhow!(
            "Password must contain at least one uppercase letter"
        ));
    }
    if !has_lower {
        return Err(anyhow::anyhow!(
            "Password must contain at least one lowercase letter"
        ));
    }
    if !has_digit {
        return Err(anyhow::anyhow!("Password must contain at least one digit"));
    }
    if !has_special {
        return Err(anyhow::anyhow!(
            "Password must contain at least one special character"
        ));
    }

    Ok(())
}

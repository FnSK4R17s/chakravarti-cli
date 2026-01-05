//! Credential storage for cloud authentication tokens.

use crate::cloud::error::CloudError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SERVICE_NAME: &str = "chakravarti-cloud";
const USERNAME: &str = "default";

/// Stored authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
}

/// Store tokens securely using system keychain or fallback to file
pub fn store_tokens(tokens: &StoredTokens) -> Result<(), CloudError> {
    // WSL/Linux often doesn't have a working keyring, so always use file storage
    // to ensure tokens are persisted reliably
    store_tokens_file(tokens)
}

/// Load tokens from storage
pub fn load_tokens() -> Result<StoredTokens, CloudError> {
    // Try keyring first
    if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, USERNAME) {
        if let Ok(json) = entry.get_password() {
            return serde_json::from_str(&json)
                .map_err(|e| CloudError::CredentialError(e.to_string()));
        }
    }
    
    // Fallback to file
    load_tokens_file()
}

/// Clear stored tokens
pub fn clear_tokens() -> Result<(), CloudError> {
    // Clear from keyring
    if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, USERNAME) {
        let _ = entry.delete_credential();
    }
    
    // Clear file storage
    let path = token_file_path()?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| CloudError::CredentialError(e.to_string()))?;
    }
    
    Ok(())
}

/// Check if tokens are stored
pub fn has_tokens() -> bool {
    load_tokens().is_ok()
}

/// Get the token file path
fn token_file_path() -> Result<PathBuf, CloudError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| CloudError::CredentialError("Could not find config directory".into()))?;
    
    let ckrv_dir = config_dir.join("chakravarti");
    if !ckrv_dir.exists() {
        fs::create_dir_all(&ckrv_dir)
            .map_err(|e| CloudError::CredentialError(e.to_string()))?;
    }
    
    Ok(ckrv_dir.join("cloud-tokens.json"))
}

/// Store tokens to file (fallback)
fn store_tokens_file(tokens: &StoredTokens) -> Result<(), CloudError> {
    let path = token_file_path()?;
    let json = serde_json::to_string_pretty(tokens)
        .map_err(|e| CloudError::CredentialError(e.to_string()))?;
    
    fs::write(&path, json)
        .map_err(|e| CloudError::CredentialError(e.to_string()))?;
    
    // Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms)
            .map_err(|e| CloudError::CredentialError(e.to_string()))?;
    }
    
    Ok(())
}

/// Load tokens from file (fallback)
fn load_tokens_file() -> Result<StoredTokens, CloudError> {
    let path = token_file_path()?;
    
    if !path.exists() {
        return Err(CloudError::NotAuthenticated);
    }
    
    let json = fs::read_to_string(&path)
        .map_err(|e| CloudError::CredentialError(e.to_string()))?;
    
    serde_json::from_str(&json)
        .map_err(|e| CloudError::CredentialError(e.to_string()))
}

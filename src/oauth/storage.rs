//! Token storage backends for persisting OAuth credentials.
//!
//! Provides [`TokenStorage`] trait with [`FileTokenStorage`] and [`MemoryTokenStorage`].
//! Storage operations are sync because we're reading/writing small JSON files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use super::error::{OAuthError, Result};
use super::token::TokenInfo;

// ── Trait ────────────────────────────────────────────────────────────

/// Trait for token storage backends.
///
/// Operations are sync (small JSON reads/writes). All implementations
/// must be `Send + Sync` for use across async tasks.
pub trait TokenStorage: Send + Sync {
    /// Load the stored token, if any.
    fn load(&self) -> Result<Option<TokenInfo>>;

    /// Save a token to storage.
    fn save(&self, token: &TokenInfo) -> Result<()>;

    /// Remove the stored token.
    fn remove(&self) -> Result<()>;
}

impl<T: TokenStorage + ?Sized> TokenStorage for std::sync::Arc<T> {
    fn load(&self) -> Result<Option<TokenInfo>> {
        (**self).load()
    }
    fn save(&self, token: &TokenInfo) -> Result<()> {
        (**self).save(token)
    }
    fn remove(&self) -> Result<()> {
        (**self).remove()
    }
}

impl<T: TokenStorage + ?Sized> TokenStorage for Box<T> {
    fn load(&self) -> Result<Option<TokenInfo>> {
        (**self).load()
    }
    fn save(&self, token: &TokenInfo) -> Result<()> {
        (**self).save(token)
    }
    fn remove(&self) -> Result<()> {
        (**self).remove()
    }
}

// ── FileTokenStorage ────────────────────────────────────────────────

/// Default config directory under user's home.
const CONFIG_DIR: &str = ".config/anthropic";

/// Default token file name.
const TOKEN_FILE: &str = "oauth-tokens.json";

/// File permissions (Unix only): owner read/write.
#[cfg(unix)]
const FILE_MODE: u32 = 0o600;

/// Directory permissions (Unix only): owner read/write/execute.
#[cfg(unix)]
const DIR_MODE: u32 = 0o700;

/// Token file structure for JSON storage.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TokenFile {
    #[serde(flatten)]
    tokens: HashMap<String, serde_json::Value>,
}

/// File-based token storage with secure permissions.
///
/// Persists to `~/.config/anthropic/oauth-tokens.json` by default.
/// Uses atomic write (temp + rename) and 0600 permissions on Unix.
#[derive(Debug, Clone)]
pub struct FileTokenStorage {
    path: PathBuf,
    /// Key in the JSON file (defaults to "anthropic").
    key: String,
}

impl FileTokenStorage {
    /// Create storage with the default path (`~/.config/anthropic/oauth-tokens.json`).
    pub fn default_path() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| OAuthError::Storage("cannot determine home directory".into()))?;
        let path = home.join(CONFIG_DIR).join(TOKEN_FILE);
        Ok(Self {
            path,
            key: "anthropic".to_string(),
        })
    }

    /// Create storage at a custom path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            key: "anthropic".to_string(),
        }
    }

    /// Get the path to the token file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn read_file(&self) -> Result<Option<TokenFile>> {
        match std::fs::read_to_string(&self.path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    return Ok(None);
                }
                let file: TokenFile = serde_json::from_str(&content).map_err(|e| {
                    OAuthError::Storage(format!(
                        "failed to parse token file '{}': {e}",
                        self.path.display()
                    ))
                })?;
                Ok(Some(file))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(OAuthError::Storage(format!(
                "failed to read token file '{}': {e}",
                self.path.display()
            ))),
        }
    }

    fn write_file(&self, file: &TokenFile) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    OAuthError::Storage(format!(
                        "failed to create directory '{}': {e}",
                        parent.display()
                    ))
                })?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = std::fs::Permissions::from_mode(DIR_MODE);
                    std::fs::set_permissions(parent, perms).map_err(|e| {
                        OAuthError::Storage(format!(
                            "failed to set directory permissions on '{}': {e}",
                            parent.display()
                        ))
                    })?;
                }
            }
        }

        let content = serde_json::to_string_pretty(file)?;

        // Atomic write: temp file + rename
        let temp_path = self.path.with_extension("tmp");
        std::fs::write(&temp_path, &content).map_err(|e| {
            OAuthError::Storage(format!(
                "failed to write temp file '{}': {e}",
                temp_path.display()
            ))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(FILE_MODE);
            std::fs::set_permissions(&temp_path, perms).map_err(|e| {
                OAuthError::Storage(format!(
                    "failed to set file permissions on '{}': {e}",
                    temp_path.display()
                ))
            })?;
        }

        if let Err(e) = std::fs::rename(&temp_path, &self.path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(OAuthError::Storage(format!(
                "failed to rename '{}' to '{}': {e}",
                temp_path.display(),
                self.path.display()
            )));
        }

        Ok(())
    }
}

impl TokenStorage for FileTokenStorage {
    fn load(&self) -> Result<Option<TokenInfo>> {
        let file = self.read_file()?;
        match file {
            Some(f) => match f.tokens.get(&self.key) {
                Some(value) => {
                    let token: TokenInfo = serde_json::from_value(value.clone()).map_err(|e| {
                        OAuthError::Storage(format!("failed to parse token: {e}"))
                    })?;
                    Ok(Some(token))
                }
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn save(&self, token: &TokenInfo) -> Result<()> {
        let mut file = self.read_file()?.unwrap_or_default();
        let value = serde_json::to_value(token)?;
        file.tokens.insert(self.key.clone(), value);
        self.write_file(&file)
    }

    fn remove(&self) -> Result<()> {
        let mut file = match self.read_file()? {
            Some(f) => f,
            None => return Ok(()),
        };

        file.tokens.remove(&self.key);

        if file.tokens.is_empty() {
            match std::fs::remove_file(&self.path) {
                Ok(()) | Err(_) => Ok(()),
            }
        } else {
            self.write_file(&file)
        }
    }
}

// ── MemoryTokenStorage ──────────────────────────────────────────────

/// In-memory token storage for testing and ephemeral use.
#[derive(Debug, Clone)]
pub struct MemoryTokenStorage {
    inner: std::sync::Arc<RwLock<Option<TokenInfo>>>,
}

impl Default for MemoryTokenStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTokenStorage {
    /// Create a new empty MemoryTokenStorage.
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(RwLock::new(None)),
        }
    }

    /// Create with an initial token.
    pub fn with_token(token: TokenInfo) -> Self {
        Self {
            inner: std::sync::Arc::new(RwLock::new(Some(token))),
        }
    }
}

impl TokenStorage for MemoryTokenStorage {
    fn load(&self) -> Result<Option<TokenInfo>> {
        let guard = self.inner.read().map_err(|e| {
            OAuthError::Storage(format!("lock poisoned: {e}"))
        })?;
        Ok(guard.clone())
    }

    fn save(&self, token: &TokenInfo) -> Result<()> {
        let mut guard = self.inner.write().map_err(|e| {
            OAuthError::Storage(format!("lock poisoned: {e}"))
        })?;
        *guard = Some(token.clone());
        Ok(())
    }

    fn remove(&self) -> Result<()> {
        let mut guard = self.inner.write().map_err(|e| {
            OAuthError::Storage(format!("lock poisoned: {e}"))
        })?;
        *guard = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // ── MemoryTokenStorage tests ────────────────────────────────────

    #[test]
    fn test_memory_save_load() {
        let storage = MemoryTokenStorage::new();
        assert!(storage.load().unwrap().is_none());

        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        storage.save(&token).unwrap();

        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "access");
    }

    #[test]
    fn test_memory_remove() {
        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        let storage = MemoryTokenStorage::with_token(token);

        assert!(storage.load().unwrap().is_some());
        storage.remove().unwrap();
        assert!(storage.load().unwrap().is_none());
    }

    #[test]
    fn test_memory_overwrite() {
        let storage = MemoryTokenStorage::new();

        let token1 = TokenInfo::new("access1".into(), "refresh1".into(), 3600);
        storage.save(&token1).unwrap();

        let token2 = TokenInfo::new("access2".into(), "refresh2".into(), 7200);
        storage.save(&token2).unwrap();

        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "access2");
    }

    #[test]
    fn test_memory_load_empty() {
        let storage = MemoryTokenStorage::new();
        assert!(storage.load().unwrap().is_none());
    }

    #[test]
    fn test_memory_remove_empty() {
        let storage = MemoryTokenStorage::new();
        storage.remove().unwrap(); // Should not error
    }

    #[test]
    fn test_memory_clone_shares_state() {
        let s1 = MemoryTokenStorage::new();
        let s2 = s1.clone();

        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        s1.save(&token).unwrap();

        let loaded = s2.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "access");
    }

    // ── Arc/Box blanket impl tests ──────────────────────────────────

    #[test]
    fn test_arc_storage() {
        let storage = Arc::new(MemoryTokenStorage::new());
        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        storage.save(&token).unwrap();
        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "access");
    }

    #[test]
    fn test_box_dyn_storage() {
        let storage: Box<dyn TokenStorage> = Box::new(MemoryTokenStorage::new());
        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        storage.save(&token).unwrap();
        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "access");
    }

    // ── FileTokenStorage tests ──────────────────────────────────────

    #[test]
    fn test_file_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tokens.json");
        let storage = FileTokenStorage::new(&path);

        assert!(storage.load().unwrap().is_none());

        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        storage.save(&token).unwrap();

        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "access");
        assert_eq!(loaded.refresh_token, "refresh");
    }

    #[test]
    fn test_file_create_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("dir").join("tokens.json");
        let storage = FileTokenStorage::new(&path);

        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        storage.save(&token).unwrap();
        assert!(path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn test_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tokens.json");
        let storage = FileTokenStorage::new(&path);

        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        storage.save(&token).unwrap();

        let metadata = std::fs::metadata(&path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn test_file_remove() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tokens.json");
        let storage = FileTokenStorage::new(&path);

        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        storage.save(&token).unwrap();
        assert!(path.exists());

        storage.remove().unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_file_remove_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        let storage = FileTokenStorage::new(&path);
        storage.remove().unwrap(); // Should not error
    }

    #[test]
    fn test_file_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tokens.json");
        let storage = FileTokenStorage::new(&path);

        let token1 = TokenInfo::new("access1".into(), "refresh1".into(), 3600);
        storage.save(&token1).unwrap();

        let token2 = TokenInfo::new("access2".into(), "refresh2".into(), 7200);
        storage.save(&token2).unwrap();

        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "access2");
    }

    #[test]
    fn test_file_atomic_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tokens.json");
        let storage = FileTokenStorage::new(&path);

        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        storage.save(&token).unwrap();

        // Temp file should not exist after successful write
        let temp_path = path.with_extension("tmp");
        assert!(!temp_path.exists());
    }
}

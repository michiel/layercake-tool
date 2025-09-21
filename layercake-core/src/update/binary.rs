use super::{BinaryManager, UpdateError};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::SystemTime;

/// Default binary manager implementation
pub struct DefaultBinaryManager;

impl DefaultBinaryManager {
    pub fn new() -> Self {
        Self
    }

    fn get_backup_path(original: &PathBuf) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let mut backup_path = original.clone();
        let file_name = original
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        
        backup_path.set_file_name(format!("{}.backup.{}", file_name, timestamp));
        backup_path
    }

    async fn calculate_sha256(path: &PathBuf) -> Result<String, UpdateError> {
        let contents = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

impl Default for DefaultBinaryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BinaryManager for DefaultBinaryManager {
    async fn verify_binary(&self, path: &PathBuf, expected_size: Option<u64>) -> Result<(), UpdateError> {
        // Check if file exists
        if !path.exists() {
            return Err(UpdateError::VerificationError(
                format!("Binary not found at: {}", path.display())
            ));
        }

        // Check if it's a file (not a directory)
        let metadata = tokio::fs::metadata(path).await?;
        if !metadata.is_file() {
            return Err(UpdateError::VerificationError(
                format!("Path is not a file: {}", path.display())
            ));
        }

        // Verify size if provided
        if let Some(expected) = expected_size {
            let actual = metadata.len();
            if actual != expected {
                return Err(UpdateError::VerificationError(
                    format!("Size mismatch: expected {}, got {}", expected, actual)
                ));
            }
        }

        // Check if file is executable (on Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 == 0 {
                return Err(UpdateError::VerificationError(
                    "Binary is not executable".to_string()
                ));
            }
        }

        // Try to read the file to ensure it's accessible
        let _contents = tokio::fs::read(path).await.map_err(|e| {
            UpdateError::VerificationError(format!("Cannot read binary: {}", e))
        })?;

        Ok(())
    }

    async fn create_backup(&self, current: &PathBuf) -> Result<PathBuf, UpdateError> {
        if !current.exists() {
            return Err(UpdateError::BackupError(
                format!("Original binary not found: {}", current.display())
            ));
        }

        let backup_path = Self::get_backup_path(current);
        
        tokio::fs::copy(current, &backup_path).await.map_err(|e| {
            UpdateError::BackupError(format!("Failed to create backup: {}", e))
        })?;

        // Verify backup was created successfully
        self.verify_binary(&backup_path, None).await.map_err(|e| {
            UpdateError::BackupError(format!("Backup verification failed: {}", e))
        })?;

        Ok(backup_path)
    }

    async fn install_binary(&self, source: &PathBuf, target: &PathBuf) -> Result<(), UpdateError> {
        // Verify source binary
        self.verify_binary(source, None).await?;

        // Ensure target directory exists
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                UpdateError::InstallationError(format!("Failed to create target directory: {}", e))
            })?;
        }

        // On Windows, we might need to handle the case where the target is currently running
        #[cfg(windows)]
        {
            if target.exists() {
                // Try to rename the existing file first (Windows locks running executables)
                let temp_name = format!("{}.old", target.display());
                let temp_path = PathBuf::from(&temp_name);
                
                if let Err(e) = tokio::fs::rename(target, &temp_path).await {
                    return Err(UpdateError::InstallationError(
                        format!("Failed to move existing binary (is it running?): {}", e)
                    ));
                }
            }
        }

        // Copy new binary to target location
        tokio::fs::copy(source, target).await.map_err(|e| {
            UpdateError::InstallationError(format!("Failed to install binary: {}", e))
        })?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(target).await?.permissions();
            perms.set_mode(0o755); // rwxr-xr-x
            tokio::fs::set_permissions(target, perms).await.map_err(|e| {
                UpdateError::InstallationError(format!("Failed to set executable permissions: {}", e))
            })?;
        }

        // Verify installation
        self.verify_binary(target, None).await.map_err(|e| {
            UpdateError::InstallationError(format!("Installation verification failed: {}", e))
        })?;

        Ok(())
    }

    async fn rollback(&self, backup: &PathBuf, target: &PathBuf) -> Result<(), UpdateError> {
        if !backup.exists() {
            return Err(UpdateError::BackupError(
                format!("Backup file not found: {}", backup.display())
            ));
        }

        // Verify backup before restoring
        self.verify_binary(backup, None).await?;

        // Restore from backup
        tokio::fs::copy(backup, target).await.map_err(|e| {
            UpdateError::BackupError(format!("Failed to restore from backup: {}", e))
        })?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(target).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(target, perms).await.map_err(|e| {
                UpdateError::BackupError(format!("Failed to set executable permissions after rollback: {}", e))
            })?;
        }

        // Verify rollback
        self.verify_binary(target, None).await.map_err(|e| {
            UpdateError::BackupError(format!("Rollback verification failed: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_verify_nonexistent_binary() {
        let manager = DefaultBinaryManager::new();
        let path = PathBuf::from("/nonexistent/path");
        
        let result = manager.verify_binary(&path, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_backup_and_rollback() {
        let manager = DefaultBinaryManager::new();
        let temp_dir = std::env::temp_dir();
        let original_path = temp_dir.join("test_binary_layercake");
        
        // Create a test binary
        tokio::fs::write(&original_path, b"test content").await
            .expect("Failed to write test content");
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&original_path).await
                .expect("Failed to read file metadata for permissions")
                .permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&original_path, perms).await
                .expect("Failed to set file permissions");
        }

        // Create backup
        let backup_path = manager.create_backup(&original_path).await
            .expect("Failed to create backup");
        assert!(backup_path.exists());

        // Modify original
        tokio::fs::write(&original_path, b"modified content").await
            .expect("Failed to modify original file");

        // Rollback
        manager.rollback(&backup_path, &original_path).await
            .expect("Failed to rollback file");

        // Verify rollback worked
        let content = tokio::fs::read_to_string(&original_path).await
            .expect("Failed to read file after rollback");
        assert_eq!(content, "test content");

        // Cleanup
        let _ = tokio::fs::remove_file(&original_path).await;
        let _ = tokio::fs::remove_file(&backup_path).await;
    }
}
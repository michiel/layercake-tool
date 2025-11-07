use super::{
    BinaryManager, PlatformDetector, PlatformInfo, ReleaseInfo, UpdateError, UpdateInfo, Updater,
    VersionComparison, VersionManager,
};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

/// Default updater implementation
pub struct DefaultUpdater {
    version_manager: Box<dyn VersionManager + Send + Sync>,
    platform_detector: Box<dyn PlatformDetector + Send + Sync>,
    binary_manager: Box<dyn BinaryManager + Send + Sync>,
    client: Client,
    temp_dir: PathBuf,
}

impl DefaultUpdater {
    pub fn new(
        version_manager: Box<dyn VersionManager + Send + Sync>,
        platform_detector: Box<dyn PlatformDetector + Send + Sync>,
        binary_manager: Box<dyn BinaryManager + Send + Sync>,
    ) -> Self {
        Self {
            version_manager,
            platform_detector,
            binary_manager,
            client: Client::new(),
            temp_dir: std::env::temp_dir(),
        }
    }

    #[allow(dead_code)] // Reserved for future custom temp directory configuration
    pub fn with_temp_dir(mut self, temp_dir: PathBuf) -> Self {
        self.temp_dir = temp_dir;
        self
    }

    fn find_asset_for_platform<'a>(
        &self,
        release: &'a ReleaseInfo,
        platform: &PlatformInfo,
    ) -> Result<&'a super::ReleaseAsset, UpdateError> {
        let pattern = self.platform_detector.get_asset_pattern(platform);

        // Try exact match first
        if let Some(asset) = release.assets.iter().find(|asset| asset.name == pattern) {
            return Ok(asset);
        }

        // Try partial matches
        let candidates: Vec<_> = release
            .assets
            .iter()
            .filter(|asset| {
                asset.name.contains(&platform.os) && asset.name.contains(&platform.arch)
            })
            .collect();

        match candidates.len() {
            0 => Err(UpdateError::VerificationError(format!(
                "No compatible asset found for platform {}-{}",
                platform.os, platform.arch
            ))),
            1 => Ok(candidates[0]),
            _ => {
                // Multiple candidates, try to find the best match
                for asset in &candidates {
                    if asset.name.ends_with(&platform.extension) {
                        return Ok(asset);
                    }
                }
                // If no exact extension match, return the first candidate
                Ok(candidates[0])
            }
        }
    }

    async fn download_with_progress(
        &self,
        url: &str,
        target_path: &PathBuf,
        expected_size: Option<u64>,
    ) -> Result<(), UpdateError> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", "layercake-updater")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(UpdateError::NetworkError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let total_size = response.content_length().or(expected_size);

        let mut file = tokio::fs::File::create(target_path).await?;
        let content = response.bytes().await?;

        file.write_all(&content).await?;
        let downloaded = content.len() as u64;

        // Progress reporting
        if let Some(total) = total_size {
            let progress = downloaded as f64 / total as f64;
            tracing::debug!("Download progress: {:.1}%", progress * 100.0);
        }

        file.sync_all().await?;

        // Verify download size if expected size was provided
        if let Some(expected) = expected_size {
            if downloaded != expected {
                return Err(UpdateError::VerificationError(format!(
                    "Download size mismatch: expected {}, got {}",
                    expected, downloaded
                )));
            }
        }

        Ok(())
    }

    fn get_current_executable() -> Result<PathBuf, UpdateError> {
        std::env::current_exe().map_err(|e| {
            UpdateError::InstallationError(format!(
                "Cannot determine current executable path: {}",
                e
            ))
        })
    }
}

#[async_trait]
impl Updater for DefaultUpdater {
    async fn check_for_updates(&self, include_prerelease: bool) -> Result<UpdateInfo, UpdateError> {
        let current_version = self.version_manager.get_current_version().await?;
        let latest_release = self
            .version_manager
            .get_latest_version(include_prerelease)
            .await?;

        let comparison = self
            .version_manager
            .compare_versions(&current_version, &latest_release.tag_name)
            .await?;

        let needs_update = comparison == VersionComparison::Newer;

        Ok(UpdateInfo {
            current_version,
            latest_version: latest_release.tag_name.clone(),
            needs_update,
            release: Some(latest_release),
        })
    }

    async fn download_release(
        &self,
        release: &ReleaseInfo,
        platform: &PlatformInfo,
    ) -> Result<PathBuf, UpdateError> {
        let asset = self.find_asset_for_platform(release, platform)?;

        let temp_filename = format!("layercake-{}-{}", release.tag_name, asset.name);
        let temp_path = self.temp_dir.join(temp_filename);

        tracing::info!(
            "Downloading {} to {}",
            asset.download_url,
            temp_path.display()
        );

        self.download_with_progress(&asset.download_url, &temp_path, Some(asset.size))
            .await?;

        // Verify the downloaded binary
        self.binary_manager
            .verify_binary(&temp_path, Some(asset.size))
            .await?;

        Ok(temp_path)
    }

    async fn perform_update(&self, force: bool, backup: bool) -> Result<(), UpdateError> {
        let platform = self.platform_detector.detect_platform()?;
        let update_info = self.check_for_updates(false).await?;

        if !update_info.needs_update && !force {
            tracing::info!("Already up to date ({})", update_info.current_version);
            return Ok(());
        }

        let release = update_info.release.as_ref().ok_or_else(|| {
            UpdateError::VersionError("No release information available".to_string())
        })?;

        tracing::info!(
            "Updating from {} to {}",
            update_info.current_version,
            update_info.latest_version
        );

        // Download the new binary
        let new_binary_path = self.download_release(release, &platform).await?;

        // Get current executable path
        let current_exe = Self::get_current_executable()?;

        // Create backup if requested
        let backup_path = if backup {
            Some(self.binary_manager.create_backup(&current_exe).await?)
        } else {
            None
        };

        // Install the new binary
        match self
            .binary_manager
            .install_binary(&new_binary_path, &current_exe)
            .await
        {
            Ok(()) => {
                tracing::info!("Successfully updated to {}", update_info.latest_version);

                // Clean up temporary file
                if let Err(e) = tokio::fs::remove_file(&new_binary_path).await {
                    tracing::warn!("Failed to clean up temporary file: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Installation failed: {}", e);

                // Attempt rollback if we have a backup
                if let Some(backup) = backup_path {
                    tracing::info!("Attempting rollback...");
                    if let Err(rollback_err) =
                        self.binary_manager.rollback(&backup, &current_exe).await
                    {
                        tracing::error!("Rollback also failed: {}", rollback_err);
                        return Err(UpdateError::InstallationError(format!(
                            "Installation failed and rollback failed: {} (rollback error: {})",
                            e, rollback_err
                        )));
                    }
                    tracing::info!("Successfully rolled back to previous version");
                }

                return Err(e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::{DefaultBinaryManager, DefaultPlatformDetector, GitHubVersionManager};

    #[tokio::test]
    async fn test_updater_creation() {
        let version_manager = Box::new(GitHubVersionManager::new(
            "michiel/layercake-tool".to_string(),
        ));
        let platform_detector = Box::new(DefaultPlatformDetector::new());
        let binary_manager = Box::new(DefaultBinaryManager::new());

        let updater = DefaultUpdater::new(version_manager, platform_detector, binary_manager);

        // Basic test that updater can be created
        assert!(!updater.temp_dir.as_os_str().is_empty());
    }
}

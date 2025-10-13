use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

pub mod binary;
pub mod command;
pub mod github;
pub mod platform;
pub mod updater;

/// Update-related errors
#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Version comparison failed: {0}")]
    VersionError(String),

    #[error("Binary verification failed: {0}")]
    VerificationError(String),

    #[error("Installation failed: {0}")]
    InstallationError(String),

    #[error("Permission denied: {0}")]
    #[allow(dead_code)] // Reserved for future permission handling
    PermissionError(String),

    #[error("Backup creation failed: {0}")]
    BackupError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Information about a GitHub release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub prerelease: bool,
    pub assets: Vec<ReleaseAsset>,
    pub published_at: Option<String>,
}

/// GitHub release asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    pub size: u64,
    pub content_type: Option<String>,
}

/// Version comparison result
#[derive(Debug, PartialEq)]
pub enum VersionComparison {
    Newer,
    Same,
    Older,
}

/// Update information
#[derive(Debug)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub needs_update: bool,
    pub release: Option<ReleaseInfo>,
}

/// Update progress information
#[derive(Debug)]
#[allow(dead_code)] // Reserved for future update progress reporting
pub struct UpdateProgress {
    pub stage: UpdateStage,
    pub progress: f64,
    pub message: String,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
}

/// Update stages
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for future update stage tracking
pub enum UpdateStage {
    CheckingVersion,
    DownloadingRelease,
    VerifyingBinary,
    CreatingBackup,
    InstallingBinary,
    CleaningUp,
    Complete,
}

/// Update channel
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for future update channel management
pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
    Custom(String),
}

/// Platform information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub extension: String,
}

/// Trait for version management
#[async_trait]
pub trait VersionManager {
    async fn get_current_version(&self) -> Result<String, UpdateError>;
    async fn get_latest_version(
        &self,
        include_prerelease: bool,
    ) -> Result<ReleaseInfo, UpdateError>;
    async fn compare_versions(
        &self,
        current: &str,
        latest: &str,
    ) -> Result<VersionComparison, UpdateError>;
}

/// Trait for platform detection
pub trait PlatformDetector {
    fn detect_platform(&self) -> Result<PlatformInfo, UpdateError>;
    fn get_asset_pattern(&self, platform: &PlatformInfo) -> String;
}

/// Trait for binary management
#[async_trait]
pub trait BinaryManager {
    async fn verify_binary(
        &self,
        path: &PathBuf,
        expected_size: Option<u64>,
    ) -> Result<(), UpdateError>;
    async fn create_backup(&self, current: &PathBuf) -> Result<PathBuf, UpdateError>;
    async fn install_binary(&self, source: &PathBuf, target: &PathBuf) -> Result<(), UpdateError>;
    async fn rollback(&self, backup: &PathBuf, target: &PathBuf) -> Result<(), UpdateError>;
}

/// Trait for the main updater
#[async_trait]
pub trait Updater {
    async fn check_for_updates(&self, include_prerelease: bool) -> Result<UpdateInfo, UpdateError>;
    async fn download_release(
        &self,
        release: &ReleaseInfo,
        platform: &PlatformInfo,
    ) -> Result<PathBuf, UpdateError>;
    async fn perform_update(&self, force: bool, backup: bool) -> Result<(), UpdateError>;
}

pub use binary::DefaultBinaryManager;
pub use github::GitHubVersionManager;
pub use platform::DefaultPlatformDetector;
pub use updater::DefaultUpdater;

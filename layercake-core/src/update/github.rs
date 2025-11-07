use super::{ReleaseAsset, ReleaseInfo, UpdateError, VersionComparison, VersionManager};
use async_trait::async_trait;
use reqwest::Client;
use semver::Version;
use serde_json::Value;

/// GitHub API client for version management
pub struct GitHubVersionManager {
    client: Client,
    repo: String,
    api_base: String,
}

impl GitHubVersionManager {
    pub fn new(repo: String) -> Self {
        Self {
            client: Client::new(),
            repo,
            api_base: "https://api.github.com".to_string(),
        }
    }

    #[allow(dead_code)] // Reserved for future custom GitHub API endpoints
    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.api_base = api_base;
        self
    }

    fn clean_version(version: &str) -> String {
        // Remove 'v' prefix if present
        version.strip_prefix('v').unwrap_or(version).to_string()
    }
}

#[async_trait]
impl VersionManager for GitHubVersionManager {
    async fn get_current_version(&self) -> Result<String, UpdateError> {
        // Get current version from Cargo.toml or binary
        let version = env!("CARGO_PKG_VERSION");
        Ok(Self::clean_version(version))
    }

    async fn get_latest_version(
        &self,
        include_prerelease: bool,
    ) -> Result<ReleaseInfo, UpdateError> {
        let url = if include_prerelease {
            format!("{}/repos/{}/releases", self.api_base, self.repo)
        } else {
            format!("{}/repos/{}/releases/latest", self.api_base, self.repo)
        };

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "layercake-updater")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(UpdateError::NetworkError(response.error_for_status().unwrap_err()));
        }

        let json: Value = response.json().await?;

        if include_prerelease {
            // Get the first release from the array
            let releases = json.as_array().ok_or_else(|| {
                UpdateError::VersionError("Expected array of releases".to_string())
            })?;

            if releases.is_empty() {
                return Err(UpdateError::VersionError("No releases found".to_string()));
            }

            self.parse_release(&releases[0])
        } else {
            self.parse_release(&json)
        }
    }

    async fn compare_versions(
        &self,
        current: &str,
        latest: &str,
    ) -> Result<VersionComparison, UpdateError> {
        let current_clean = Self::clean_version(current);
        let latest_clean = Self::clean_version(latest);

        let current_version = Version::parse(&current_clean).map_err(|e| {
            UpdateError::VersionError(format!(
                "Invalid current version '{}': {}",
                current_clean, e
            ))
        })?;

        let latest_version = Version::parse(&latest_clean).map_err(|e| {
            UpdateError::VersionError(format!("Invalid latest version '{}': {}", latest_clean, e))
        })?;

        Ok(match current_version.cmp(&latest_version) {
            std::cmp::Ordering::Less => VersionComparison::Newer,
            std::cmp::Ordering::Equal => VersionComparison::Same,
            std::cmp::Ordering::Greater => VersionComparison::Older,
        })
    }
}

impl GitHubVersionManager {
    fn parse_release(&self, json: &Value) -> Result<ReleaseInfo, UpdateError> {
        let tag_name = json["tag_name"]
            .as_str()
            .ok_or_else(|| UpdateError::VersionError("Missing tag_name".to_string()))?
            .to_string();

        let name = json["name"].as_str().unwrap_or(&tag_name).to_string();

        let body = json["body"].as_str().map(|s| s.to_string());

        let prerelease = json["prerelease"].as_bool().unwrap_or(false);

        let published_at = json["published_at"].as_str().map(|s| s.to_string());

        let assets = json["assets"]
            .as_array()
            .ok_or_else(|| UpdateError::VersionError("Missing assets array".to_string()))?
            .iter()
            .map(|asset| self.parse_asset(asset))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ReleaseInfo {
            tag_name,
            name,
            body,
            prerelease,
            assets,
            published_at,
        })
    }

    fn parse_asset(&self, json: &Value) -> Result<ReleaseAsset, UpdateError> {
        let name = json["name"]
            .as_str()
            .ok_or_else(|| UpdateError::VersionError("Missing asset name".to_string()))?
            .to_string();

        let download_url = json["browser_download_url"]
            .as_str()
            .ok_or_else(|| UpdateError::VersionError("Missing download URL".to_string()))?
            .to_string();

        let size = json["size"]
            .as_u64()
            .ok_or_else(|| UpdateError::VersionError("Missing asset size".to_string()))?;

        let content_type = json["content_type"].as_str().map(|s| s.to_string());

        Ok(ReleaseAsset {
            name,
            download_url,
            size,
            content_type,
        })
    }
}

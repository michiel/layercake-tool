use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;

use super::{
    BinaryManager, DefaultBinaryManager, DefaultPlatformDetector, DefaultUpdater, GitHubVersionManager,
    UpdateError, Updater,
};

pub struct UpdateCommand {
    pub check_only: bool,
    pub force: bool,
    pub pre_release: bool,
    #[allow(dead_code)] // Reserved for future version targeting
    pub version: Option<String>,
    #[allow(dead_code)] // Reserved for future custom install directory
    pub install_dir: Option<PathBuf>,
    pub backup: bool,
    pub rollback: bool,
    pub dry_run: bool,
    #[allow(dead_code)] // Reserved for future signature verification
    pub skip_verify: bool,
}

impl UpdateCommand {
    pub async fn execute(self) -> Result<()> {
        if self.rollback {
            return self.execute_rollback().await;
        }

        let version_manager = Box::new(GitHubVersionManager::new("michiel/layercake-tool".to_string()));
        let platform_detector = Box::new(DefaultPlatformDetector::new());
        let binary_manager = Box::new(DefaultBinaryManager::new());

        let updater = DefaultUpdater::new(version_manager, platform_detector, binary_manager);

        if self.check_only || self.dry_run {
            self.check_for_updates(&updater).await
        } else {
            self.perform_update(&updater).await
        }
    }

    async fn check_for_updates(&self, updater: &DefaultUpdater) -> Result<()> {
        println!("{}", "Checking for updates...".blue());

        let update_info = updater
            .check_for_updates(self.pre_release)
            .await
            .context("Failed to check for updates")?;

        println!("Current version: {}", update_info.current_version.green());
        println!("Latest version:  {}", update_info.latest_version.cyan());

        if update_info.needs_update || self.force {
            println!("\n{}", "Update available!".green().bold());
            
            if let Some(release) = &update_info.release {
                if let Some(body) = &release.body {
                    if !body.trim().is_empty() {
                        println!("\nRelease notes:");
                        println!("{}", body.trim());
                    }
                }
            }

            if self.dry_run {
                println!("\n{}", "This is a dry run - no changes will be made.".yellow());
                println!("Run without --dry-run to perform the update.");
            } else if self.check_only {
                println!("\nRun {} to update.", "layercake update".green());
            }
        } else {
            println!("\n{}", "Already up to date!".green());
        }

        Ok(())
    }

    async fn perform_update(&self, updater: &DefaultUpdater) -> Result<()> {
        println!("{}", "Starting update process...".blue());

        match updater.perform_update(self.force, self.backup).await {
            Ok(()) => {
                println!("\n{}", "✓ Update completed successfully!".green().bold());
                println!("You may need to restart any running layercake processes.");
            }
            Err(UpdateError::VersionError(msg)) if msg.contains("Already up to date") => {
                println!("\n{}", "Already up to date!".green());
                if self.force {
                    println!("Use {} to force update anyway.", "--force".yellow());
                }
            }
            Err(e) => {
                eprintln!("\n{} {}", "✗ Update failed:".red().bold(), e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    async fn execute_rollback(&self) -> Result<()> {
        println!("{}", "Rolling back to previous version...".blue());

        // Find backup files
        let current_exe = std::env::current_exe()
            .context("Cannot determine current executable path")?;

        let parent_dir = current_exe.parent()
            .context("Cannot determine executable directory")?;

        // Look for backup files
        let mut backup_files = Vec::new();
        let mut entries = tokio::fs::read_dir(parent_dir).await
            .context("Cannot read executable directory")?;

        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            if file_name_str.starts_with("layercake") && file_name_str.contains(".backup.") {
                backup_files.push(entry.path());
            }
        }

        if backup_files.is_empty() {
            eprintln!("{}", "No backup files found.".red());
            eprintln!("Create a backup with {} before updating.", "layercake update --backup".yellow());
            return Ok(());
        }

        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| {
            let a_modified = std::fs::metadata(a)
                .and_then(|meta| meta.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_modified = std::fs::metadata(b)
                .and_then(|meta| meta.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_modified.cmp(&a_modified)
        });

        let latest_backup = &backup_files[0];
        println!("Found backup: {}", latest_backup.display());

        let binary_manager = DefaultBinaryManager::new();
        
        match binary_manager.rollback(latest_backup, &current_exe).await {
            Ok(()) => {
                println!("\n{}", "✓ Rollback completed successfully!".green().bold());
            }
            Err(e) => {
                eprintln!("\n{} {}", "✗ Rollback failed:".red().bold(), e);
                return Err(e.into());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_command_creation() {
        let cmd = UpdateCommand {
            check_only: true,
            force: false,
            pre_release: false,
            version: None,
            install_dir: None,
            backup: false,
            rollback: false,
            dry_run: false,
            skip_verify: false,
        };

        // Basic test that command can be created
        assert!(cmd.check_only);
        assert!(!cmd.force);
    }
}
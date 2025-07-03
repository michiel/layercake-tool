use super::{PlatformDetector, PlatformInfo, UpdateError};

/// Default platform detector
pub struct DefaultPlatformDetector;

impl DefaultPlatformDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultPlatformDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformDetector for DefaultPlatformDetector {
    fn detect_platform(&self) -> Result<PlatformInfo, UpdateError> {
        let os = detect_os()?;
        let arch = detect_arch()?;
        let extension = get_extension(&os);

        Ok(PlatformInfo { os, arch, extension })
    }

    fn get_asset_pattern(&self, platform: &PlatformInfo) -> String {
        format!("layercake-{}-{}{}", platform.os, platform.arch, platform.extension)
    }
}

fn detect_os() -> Result<String, UpdateError> {
    match std::env::consts::OS {
        "linux" => Ok("linux".to_string()),
        "macos" => Ok("macos".to_string()),
        "windows" => Ok("windows".to_string()),
        other => Err(UpdateError::VersionError(format!(
            "Unsupported operating system: {}",
            other
        ))),
    }
}

fn detect_arch() -> Result<String, UpdateError> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("x86_64".to_string()),
        "aarch64" => Ok("aarch64".to_string()),
        "arm64" => Ok("aarch64".to_string()), // macOS uses arm64 but we normalize to aarch64
        other => Err(UpdateError::VersionError(format!(
            "Unsupported architecture: {}",
            other
        ))),
    }
}

fn get_extension(os: &str) -> String {
    match os {
        "windows" => ".exe".to_string(),
        _ => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let detector = DefaultPlatformDetector::new();
        let platform = detector.detect_platform()
            .expect("Platform detection should work on supported systems");
        
        // Basic sanity checks
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
        
        #[cfg(target_os = "windows")]
        assert_eq!(platform.extension, ".exe");
        
        #[cfg(not(target_os = "windows"))]
        assert_eq!(platform.extension, "");
    }

    #[test]
    fn test_asset_pattern() {
        let detector = DefaultPlatformDetector::new();
        let platform = PlatformInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            extension: "".to_string(),
        };
        
        let pattern = detector.get_asset_pattern(&platform);
        assert_eq!(pattern, "layercake-linux-x86_64");
    }
}
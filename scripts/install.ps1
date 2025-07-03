# Layercake Installation Script for Windows PowerShell
# This script downloads and installs the latest Layercake release for Windows

param(
    [string]$InstallDir = "$env:USERPROFILE\.local\bin",
    [switch]$Help,
    [switch]$Version
)

# Set error action preference
$ErrorActionPreference = "Stop"

# Configuration
$Repo = "michiel/layercake-tool"
$GitHubApi = "https://api.github.com/repos/$Repo"
$GitHubReleases = "$GitHubApi/releases/latest"
$BinaryName = "layercake.exe"

# Colors for output (if supported)
$Colors = @{
    Red = "Red"
    Green = "Green"
    Yellow = "Yellow"
    Blue = "Blue"
    White = "White"
}

# Logging functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $Colors.Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Colors.Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Colors.Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Colors.Red
}

# Show help
function Show-Help {
    Write-Host "Layercake Installation Script for Windows PowerShell"
    Write-Host ""
    Write-Host "Usage: .\install.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "This script downloads and installs the latest Layercake release"
    Write-Host "from GitHub for Windows."
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -InstallDir <path>    Custom installation directory (default: $env:USERPROFILE\.local\bin)"
    Write-Host "  -Help                 Show this help message"
    Write-Host "  -Version              Show script version"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\install.ps1                                    # Install to default location"
    Write-Host "  .\install.ps1 -InstallDir C:\tools\bin          # Install to custom directory"
    Write-Host ""
    Write-Host "Remote execution:"
    Write-Host "  irm https://raw.githubusercontent.com/michiel/layercake-tool/master/scripts/install.ps1 | iex"
    Write-Host ""
}

# Show version
function Show-Version {
    Write-Host "Layercake Installation Script v1.0.0 for Windows PowerShell"
}

# Detect platform and architecture
function Get-PlatformInfo {
    $os = "windows"
    
    # Detect architecture
    $arch = switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" { "x86_64" }
        "ARM64" { "aarch64" }
        "x86" { "i686" }
        default { 
            Write-Error "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE"
            exit 1
        }
    }
    
    Write-Info "Detected platform: $os-$arch"
    
    return @{
        OS = $os
        Arch = $arch
        Platform = "$os-$arch"
    }
}

# Check if required tools are available
function Test-Dependencies {
    $missingDeps = @()
    
    # Check for PowerShell 5.1+ (Invoke-RestMethod, Expand-Archive)
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        $missingDeps += "PowerShell 5.1 or later"
    }
    
    # Test Invoke-RestMethod
    try {
        $null = Get-Command Invoke-RestMethod -ErrorAction Stop
    } catch {
        $missingDeps += "Invoke-RestMethod cmdlet"
    }
    
    # Test Expand-Archive
    try {
        $null = Get-Command Expand-Archive -ErrorAction Stop
    } catch {
        $missingDeps += "Expand-Archive cmdlet"
    }
    
    if ($missingDeps.Count -gt 0) {
        Write-Error "Missing required dependencies: $($missingDeps -join ', ')"
        Write-Info "Please upgrade to PowerShell 5.1 or later"
        Write-Info "Download from: https://aka.ms/powershell"
        exit 1
    }
}

# Get the latest release information
function Get-LatestRelease {
    Write-Info "Fetching latest release information..."
    
    try {
        $releaseData = Invoke-RestMethod -Uri $GitHubReleases -UseBasicParsing
    } catch {
        Write-Error "Failed to fetch release information from GitHub: $($_.Exception.Message)"
        exit 1
    }
    
    if (-not $releaseData.tag_name) {
        Write-Error "No releases found in the repository"
        exit 1
    }
    
    Write-Info "Latest version: $($releaseData.tag_name)"
    
    return $releaseData
}

# Find the appropriate asset for Windows
function Find-Asset {
    param([object]$ReleaseData, [string]$Platform)
    
    Write-Info "Looking for $Platform release asset..."
    
    $version = $ReleaseData.tag_name
    $versionNoV = $version -replace '^v', ''
    
    # Common naming patterns for Windows release assets
    $patterns = @(
        "layercake-$version-windows-*.zip",
        "layercake-windows-*.zip",
        "layercake-$versionNoV-windows-*.zip",
        "layercake_${versionNoV}_windows_*.zip",
        "windows-*.zip",
        "*windows*.zip"
    )
    
    $assets = $ReleaseData.assets
    
    # Search for matching assets
    foreach ($pattern in $patterns) {
        foreach ($asset in $assets) {
            if ($asset.name -like $pattern) {
                Write-Success "Found release asset: $($asset.name)"
                return @{
                    Name = $asset.name
                    DownloadUrl = $asset.browser_download_url
                }
            }
        }
    }
    
    # If no exact match, try partial matching
    Write-Warning "Exact match not found, trying partial matching..."
    foreach ($asset in $assets) {
        if ($asset.name -like "*windows*" -and $asset.name -like "*.zip") {
            Write-Warning "Using partial match: $($asset.name)"
            return @{
                Name = $asset.name
                DownloadUrl = $asset.browser_download_url
            }
        }
    }
    
    Write-Error "No suitable release asset found for Windows"
    Write-Info "Available assets:"
    foreach ($asset in $assets) {
        Write-Host "  $($asset.name)"
    }
    exit 1
}

# Download and extract the release
function Install-LayercakeBinary {
    param([object]$Asset, [string]$InstallDirectory)
    
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
    $archivePath = Join-Path $tempDir $Asset.Name
    
    try {
        Write-Info "Downloading $($Asset.Name)..."
        
        # Download with progress
        $ProgressPreference = 'Continue'
        Invoke-WebRequest -Uri $Asset.DownloadUrl -OutFile $archivePath -UseBasicParsing
        
        Write-Info "Extracting archive..."
        
        # Extract zip file
        try {
            Expand-Archive -Path $archivePath -DestinationPath $tempDir -Force
        } catch {
            Write-Error "Failed to extract zip archive: $($_.Exception.Message)"
            return $false
        }
        
        # Find the binary in the extracted files
        $binaryPath = $null
        $searchNames = @("layercake.exe", "layercake", "layercake-windows-*.exe")
        
        foreach ($name in $searchNames) {
            $found = Get-ChildItem -Path $tempDir -Filter $name -Recurse -File | Select-Object -First 1
            if ($found) {
                $binaryPath = $found.FullName
                break
            }
        }
        
        if (-not $binaryPath) {
            Write-Error "Could not find layercake binary in extracted archive"
            Write-Info "Archive contents:"
            Get-ChildItem -Path $tempDir -Recurse -File | ForEach-Object { Write-Host "  $($_.FullName)" }
            return $false
        }
        
        Write-Success "Found binary at: $binaryPath"
        
        # Create installation directory if it doesn't exist
        if (-not (Test-Path $InstallDirectory)) {
            Write-Info "Creating directory: $InstallDirectory"
            New-Item -ItemType Directory -Path $InstallDirectory -Force | Out-Null
        }
        
        # Copy binary to installation directory
        $targetPath = Join-Path $InstallDirectory $BinaryName
        
        try {
            Copy-Item -Path $binaryPath -Destination $targetPath -Force
        } catch {
            Write-Error "Failed to copy binary to $targetPath : $($_.Exception.Message)"
            return $false
        }
        
        Write-Success "Layercake installed to $targetPath"
        return $true
        
    } finally {
        # Cleanup
        if (Test-Path $tempDir) {
            Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

# Check if the installation directory is in PATH
function Test-PathConfiguration {
    param([string]$InstallDirectory)
    
    $pathDirs = $env:PATH -split ';'
    $installDirResolved = Resolve-Path $InstallDirectory -ErrorAction SilentlyContinue
    
    $inPath = $false
    foreach ($dir in $pathDirs) {
        if ($dir -eq $InstallDirectory -or 
            ($installDirResolved -and $dir -eq $installDirResolved.Path)) {
            $inPath = $true
            break
        }
    }
    
    if ($inPath) {
        Write-Success "$InstallDirectory is already in your PATH"
        return $true
    }
    
    Write-Warning "$InstallDirectory is not in your PATH"
    Write-Info "To add it to your PATH, run the following commands in an elevated PowerShell:"
    Write-Info ""
    Write-Host "# Add to user PATH (recommended):" -ForegroundColor Yellow
    Write-Host "`$userPath = [Environment]::GetEnvironmentVariable('PATH', 'User')" -ForegroundColor Gray
    Write-Host "`$newPath = `$userPath + ';$InstallDirectory'" -ForegroundColor Gray
    Write-Host "[Environment]::SetEnvironmentVariable('PATH', `$newPath, 'User')" -ForegroundColor Gray
    Write-Info ""
    Write-Host "# Or add to system PATH (requires admin):" -ForegroundColor Yellow
    Write-Host "`$systemPath = [Environment]::GetEnvironmentVariable('PATH', 'Machine')" -ForegroundColor Gray
    Write-Host "`$newPath = `$systemPath + ';$InstallDirectory'" -ForegroundColor Gray
    Write-Host "[Environment]::SetEnvironmentVariable('PATH', `$newPath, 'Machine')" -ForegroundColor Gray
    Write-Info ""
    Write-Info "Then restart your PowerShell session"
    
    return $false
}

# Verify installation
function Test-Installation {
    param([string]$InstallDirectory)
    
    $binaryPath = Join-Path $InstallDirectory $BinaryName
    
    if (-not (Test-Path $binaryPath)) {
        Write-Error "Installation verification failed: binary not found at $binaryPath"
        return $false
    }
    
    Write-Info "Testing layercake installation..."
    
    # Test if the binary runs and shows version/help
    try {
        $output = & $binaryPath --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Layercake installation verified successfully!"
            Write-Info "Installed version: $output"
            return $true
        }
    } catch {
        # Try help instead
        try {
            $null = & $binaryPath --help 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Layercake installation verified successfully!"
                return $true
            }
        } catch {
            # Ignore
        }
    }
    
    Write-Warning "Binary installed but failed to run properly"
    Write-Info "This might be due to missing system dependencies"
    return $false
}

# Show usage information
function Show-Usage {
    Write-Info "Layercake has been installed! Here's how to use it:"
    Write-Host ""
    Write-Host "  Process a plan file:"
    Write-Host "    layercake -p <plan.yaml>"
    Write-Host ""
    Write-Host "  Watch mode for development:"
    Write-Host "    layercake -p <plan.yaml> -w"
    Write-Host ""
    Write-Host "  Get help:"
    Write-Host "    layercake --help"
    Write-Host ""
    Write-Host "  Check version:"
    Write-Host "    layercake --version"
    Write-Host ""
    Write-Info "For more information, visit: https://github.com/$Repo"
}

# Handle script parameters
if ($Help) {
    Show-Help
    exit 0
}

if ($Version) {
    Show-Version
    exit 0
}

# Main installation function
function Install-Layercake {
    Write-Info "🎂 Layercake Installation Script for Windows"
    Write-Info "============================================"
    Write-Host ""
    
    # Perform installation steps
    $platformInfo = Get-PlatformInfo
    Test-Dependencies
    $releaseData = Get-LatestRelease
    $asset = Find-Asset -ReleaseData $releaseData -Platform $platformInfo.Platform
    
    if (-not (Install-LayercakeBinary -Asset $asset -InstallDirectory $InstallDir)) {
        Write-Error "Installation failed"
        exit 1
    }
    
    # Check PATH and verify installation
    $pathOk = Test-PathConfiguration -InstallDirectory $InstallDir
    
    Write-Host ""
    if (Test-Installation -InstallDirectory $InstallDir) {
        Write-Host ""
        Write-Success "🎉 Layercake installation completed successfully!"
        
        if ($pathOk) {
            Show-Usage
        } else {
            Write-Info "Add $InstallDir to your PATH to use layercake from anywhere"
        }
    } else {
        Write-Error "Installation completed but verification failed"
        Write-Info "The binary is installed at $(Join-Path $InstallDir $BinaryName)"
        exit 1
    }
}

# Run the main installation
Install-Layercake
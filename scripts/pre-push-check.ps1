#!/usr/bin/env pwsh
# Pre-push validation script - replicates CI/CD checks locally

param(
    [switch]$SkipPrettier,
    [switch]$CheckTagVersion
)

$ErrorActionPreference = "Stop"
$script:HasErrors = $false

function Write-Step {
    param([string]$Message)
    Write-Host "`n=== $Message ===" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Failure {
    param([string]$Message)
    Write-Host "✗ $Message" -ForegroundColor Red
    $script:HasErrors = $true
}

# 1. Prettier check
if (-not $SkipPrettier) {
    Write-Step "Check Formatting (Prettier)"
    try {
        $result = npx prettier "**/*.{md,yml}" --check 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Prettier formatting OK"
        } else {
            Write-Failure "Prettier found formatting issues:"
            Write-Host $result
        }
    } catch {
        Write-Failure "Prettier check failed: $_"
    }
} else {
    Write-Host "`nSkipping Prettier check" -ForegroundColor Yellow
}

# 2. Rust formatting
Write-Step "Check Formatting (Rust)"
cargo fmt -- --check
if ($LASTEXITCODE -eq 0) {
    Write-Success "Rust formatting OK"
} else {
    Write-Failure "Rust formatting issues found. Run 'cargo fmt' to fix."
}

# 3. Clippy linting
Write-Step "Check Linting (Clippy)"
cargo clippy -- -D warnings
if ($LASTEXITCODE -eq 0) {
    Write-Success "Clippy linting OK"
} else {
    Write-Failure "Clippy found issues"
}

# 4. Tests
Write-Step "Run Tests"
cargo test --verbose
if ($LASTEXITCODE -eq 0) {
    Write-Success "All tests passed"
} else {
    Write-Failure "Some tests failed"
}

# 5. Tag version check (optional, for releases)
if ($CheckTagVersion) {
    Write-Step "Check Tag Version Match"
    try {
        $tagVersion = (git describe --tags --abbrev=0 2>$null) -replace '^v', ''
        $cargoVersion = (Select-String -Path ".\Cargo.toml" -Pattern "^version" | 
            Select-Object -First 1).Line -replace '.*"([^"]+)".*', '$1'
        
        if ($tagVersion -eq $cargoVersion) {
            Write-Success "Tag version ($tagVersion) matches Cargo.toml ($cargoVersion)"
        } else {
            Write-Failure "Version mismatch! Tag: $tagVersion, Cargo.toml: $cargoVersion"
        }
    } catch {
        Write-Host "No tags found or error reading version" -ForegroundColor Yellow
    }
}

# Summary
Write-Host "`n" + ("=" * 50) -ForegroundColor Cyan
if ($script:HasErrors) {
    Write-Host "FAILED - Fix the issues above before pushing" -ForegroundColor Red
    exit 1
} else {
    Write-Host "ALL CHECKS PASSED - Ready to push!" -ForegroundColor Green
    exit 0
}

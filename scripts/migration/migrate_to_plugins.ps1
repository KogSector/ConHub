# Migration script for Windows PowerShell
# Migrates from MCP services to the new plugin system

param(
    [string]$Config = "migration_config.json",
    [switch]$DryRun,
    [switch]$Verbose,
    [switch]$Help
)

if ($Help) {
    Write-Host @"
ConHub Plugin Migration Script

USAGE:
    .\migrate_to_plugins.ps1 [OPTIONS]

OPTIONS:
    -Config <file>    Migration configuration file (default: migration_config.json)
    -DryRun          Run in dry-run mode (no changes will be made)
    -Verbose         Enable verbose logging
    -Help            Show this help message

EXAMPLES:
    .\migrate_to_plugins.ps1                    # Run migration with default settings
    .\migrate_to_plugins.ps1 -DryRun           # Preview changes without applying them
    .\migrate_to_plugins.ps1 -Verbose          # Run with detailed logging
    .\migrate_to_plugins.ps1 -Config custom.json  # Use custom configuration file

DESCRIPTION:
    This script migrates your ConHub installation from the old MCP microservices
    architecture to the new unified plugin system. It will:
    
    1. Create a backup of your current configuration
    2. Extract service configurations from docker-compose.yml and .env files
    3. Create new plugin configurations
    4. Load the configurations into the plugins service
    5. Validate that the migration was successful
    
    The script can be run in dry-run mode to preview changes before applying them.
"@
    exit 0
}

# Set error action preference
$ErrorActionPreference = "Stop"

# Function to write colored output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$timestamp] $Message" -ForegroundColor $Color
}

function Write-Info {
    param([string]$Message)
    Write-ColorOutput $Message "Cyan"
}

function Write-Success {
    param([string]$Message)
    Write-ColorOutput $Message "Green"
}

function Write-Warning {
    param([string]$Message)
    Write-ColorOutput $Message "Yellow"
}

function Write-Error {
    param([string]$Message)
    Write-ColorOutput $Message "Red"
}

# Function to check prerequisites
function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    # Check if Python is installed
    try {
        $pythonVersion = python --version 2>&1
        Write-Info "Found Python: $pythonVersion"
    }
    catch {
        Write-Error "Python is not installed or not in PATH"
        Write-Error "Please install Python 3.7+ and try again"
        return $false
    }
    
    # Check if required Python packages are available
    $requiredPackages = @("requests")
    foreach ($package in $requiredPackages) {
        try {
            python -c "import $package" 2>$null
            Write-Info "Found Python package: $package"
        }
        catch {
            Write-Warning "Python package '$package' not found, attempting to install..."
            try {
                pip install $package
                Write-Success "Installed Python package: $package"
            }
            catch {
                Write-Error "Failed to install Python package: $package"
                Write-Error "Please run: pip install $package"
                return $false
            }
        }
    }
    
    # Check if docker-compose.yml exists
    if (-not (Test-Path "docker-compose.yml")) {
        Write-Warning "docker-compose.yml not found in current directory"
        Write-Warning "Make sure you're running this script from the ConHub root directory"
    }
    
    # Check if plugins service is running
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:3020/health" -TimeoutSec 5 -ErrorAction SilentlyContinue
        Write-Success "Plugins service is running and accessible"
    }
    catch {
        Write-Warning "Plugins service is not accessible at http://localhost:3020"
        Write-Warning "Make sure the plugins service is running before migration"
        Write-Warning "You can start it with: docker-compose up plugins"
    }
    
    return $true
}

# Function to run the Python migration script
function Invoke-Migration {
    Write-Info "Starting migration process..."
    
    $pythonScript = "scripts\migrate_to_plugins.py"
    
    if (-not (Test-Path $pythonScript)) {
        Write-Error "Migration script not found: $pythonScript"
        return $false
    }
    
    # Build Python command arguments
    $pythonArgs = @($pythonScript, "--config", $Config)
    
    if ($DryRun) {
        $pythonArgs += "--dry-run"
        Write-Info "Running in DRY RUN mode - no changes will be made"
    }
    
    if ($Verbose) {
        $pythonArgs += "--verbose"
    }
    
    try {
        Write-Info "Executing: python $($pythonArgs -join ' ')"
        $result = & python $pythonArgs
        
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Migration completed successfully!"
            return $true
        }
        else {
            Write-Error "Migration failed with exit code: $LASTEXITCODE"
            return $false
        }
    }
    catch {
        Write-Error "Failed to execute migration script: $_"
        return $false
    }
}

# Function to display post-migration instructions
function Show-PostMigrationInstructions {
    Write-Info "Post-migration instructions:"
    Write-Host ""
    Write-Host "1. Verify that all plugins are running:" -ForegroundColor Yellow
    Write-Host "   curl http://localhost:3020/api/status" -ForegroundColor Gray
    Write-Host ""
    Write-Host "2. Test plugin functionality:" -ForegroundColor Yellow
    Write-Host "   # List source plugins" -ForegroundColor Gray
    Write-Host "   curl http://localhost:3020/api/sources" -ForegroundColor Gray
    Write-Host "   # List agent plugins" -ForegroundColor Gray
    Write-Host "   curl http://localhost:3020/api/agents" -ForegroundColor Gray
    Write-Host ""
    Write-Host "3. Update your application to use the new plugins API:" -ForegroundColor Yellow
    Write-Host "   - Replace MCP service calls with plugins API calls" -ForegroundColor Gray
    Write-Host "   - Update frontend to use http://localhost:3020/api/*" -ForegroundColor Gray
    Write-Host ""
    Write-Host "4. Once everything is working, you can remove old services:" -ForegroundColor Yellow
    Write-Host "   - Comment out old MCP services in docker-compose.yml" -ForegroundColor Gray
    Write-Host "   - Remove old environment variables" -ForegroundColor Gray
    Write-Host ""
    Write-Host "5. Backup files are stored in: migration_backup/" -ForegroundColor Yellow
    Write-Host ""
}

# Main execution
function Main {
    Write-Info "ConHub Plugin Migration Script"
    Write-Info "=============================="
    
    # Check prerequisites
    if (-not (Test-Prerequisites)) {
        Write-Error "Prerequisites check failed. Please resolve the issues above and try again."
        exit 1
    }
    
    # Confirm migration (unless dry run)
    if (-not $DryRun) {
        Write-Warning "This will migrate your ConHub installation to the new plugin system."
        Write-Warning "A backup will be created, but please ensure you have your own backups."
        $confirmation = Read-Host "Do you want to continue? (y/N)"
        
        if ($confirmation -notmatch "^[Yy]") {
            Write-Info "Migration cancelled by user."
            exit 0
        }
    }
    
    # Run migration
    $success = Invoke-Migration
    
    if ($success) {
        if (-not $DryRun) {
            Show-PostMigrationInstructions
        }
        else {
            Write-Info "Dry run completed. Review the output above and run without -DryRun to apply changes."
        }
    }
    else {
        Write-Error "Migration failed. Check the output above for details."
        Write-Error "Your original configuration has not been modified."
        exit 1
    }
}

# Run main function
try {
    Main
}
catch {
    Write-Error "An unexpected error occurred: $_"
    exit 1
}
#!/usr/bin/env pwsh
<#
.SYNOPSIS
    View real-time Docker build logs for ConHub services
.DESCRIPTION
    This script displays real-time logs from currently running Docker builds
.PARAMETER Service
    Specific service to monitor (default: shows all)
.PARAMETER Follow
    Follow logs in real-time (default: true)
.EXAMPLE
    .\view-build-logs.ps1
    .\view-build-logs.ps1 -Service backend
#>

param(
    [string]$Service = "",
    [switch]$Follow = $true
)

# Color definitions
$Colors = @{
    Backend = "Green"
    Auth = "Blue" 
    Billing = "Yellow"
    Security = "Red"
    Data = "Magenta"
    AI = "Cyan"
    Webhook = "White"
    Plugins = "Gray"
    Default = "White"
}

# Function to get service color
function Get-ServiceColor {
    param([string]$ServiceName)
    return $Colors[$ServiceName] ?? $Colors.Default
}

# Function to display header
function Show-Header {
    Clear-Host
    Write-Host "=" * 100 -ForegroundColor Magenta
    Write-Host "ConHub Docker Build Logs - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Magenta
    Write-Host "=" * 100 -ForegroundColor Magenta
    Write-Host ""
}

# Function to get running Docker Compose processes
function Get-RunningBuilds {
    try {
        # Get all docker-compose processes
        $processes = Get-Process -Name "docker-compose" -ErrorAction SilentlyContinue
        
        if ($processes) {
            Write-Host "Found $($processes.Count) running docker-compose process(es)" -ForegroundColor Green
            return $processes
        } else {
            Write-Host "No running docker-compose processes found" -ForegroundColor Yellow
            return @()
        }
    }
    catch {
        Write-Host "Error checking for running builds: $_" -ForegroundColor Red
        return @()
    }
}

# Function to show current build status using docker-compose
function Show-BuildStatus {
    Write-Host "Current Docker Compose Status:" -ForegroundColor Cyan
    Write-Host ("-" * 50) -ForegroundColor Cyan
    
    try {
        # Show current docker-compose processes
        $result = docker-compose ps 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host $result
        } else {
            Write-Host "No docker-compose services currently running" -ForegroundColor Yellow
        }
    }
    catch {
        Write-Host "Error getting docker-compose status: $_" -ForegroundColor Red
    }
    
    Write-Host ""
}

# Function to monitor specific service logs
function Watch-ServiceLogs {
    param([string]$ServiceName)
    
    Write-Host "Monitoring logs for service: $ServiceName" -ForegroundColor (Get-ServiceColor $ServiceName)
    Write-Host ("-" * 50) -ForegroundColor Gray
    
    try {
        if ($Follow) {
            # Follow logs in real-time
            docker-compose logs -f $ServiceName
        } else {
            # Show current logs
            docker-compose logs $ServiceName
        }
    }
    catch {
        Write-Host "Error monitoring logs for $ServiceName`: $_" -ForegroundColor Red
    }
}

# Function to monitor all service logs
function Watch-AllLogs {
    Write-Host "Monitoring logs for all services..." -ForegroundColor Green
    Write-Host ("-" * 50) -ForegroundColor Gray
    
    try {
        if ($Follow) {
            # Follow all logs with service names and colors
            docker-compose logs -f --tail=50
        } else {
            # Show current logs for all services
            docker-compose logs --tail=100
        }
    }
    catch {
        Write-Host "Error monitoring all logs: $_" -ForegroundColor Red
    }
}

# Function to show build progress summary
function Show-BuildProgress {
    $services = @("backend", "auth", "billing", "security", "data", "ai", "webhook", "plugins")
    
    Write-Host "Build Progress Summary:" -ForegroundColor Cyan
    Write-Host ("-" * 60) -ForegroundColor Cyan
    
    foreach ($svc in $services) {
        try {
            # Check if image exists (built successfully)
            $imageCheck = docker images "conhub-$svc" --format "table {{.Repository}}:{{.Tag}}" 2>$null
            
            if ($imageCheck -and $imageCheck -notmatch "REPOSITORY") {
                Write-Host "✅ $svc - Built successfully" -ForegroundColor Green
            } else {
                # Check if currently building
                $buildCheck = docker-compose ps $svc 2>$null
                if ($buildCheck) {
                    Write-Host "🔄 $svc - Currently building..." -ForegroundColor Yellow
                } else {
                    Write-Host "⏳ $svc - Not started" -ForegroundColor Gray
                }
            }
        }
        catch {
            Write-Host "❌ $svc - Status unknown" -ForegroundColor Red
        }
    }
    Write-Host ""
}

# Main execution
Show-Header

# Check if Docker is running
try {
    $null = docker info 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Docker is not running. Please start Docker Desktop." -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Docker is running" -ForegroundColor Green
}
catch {
    Write-Host "❌ Docker is not available: $_" -ForegroundColor Red
    exit 1
}

# Show current status
Show-BuildProgress
Show-BuildStatus

# Check for running builds
$runningBuilds = Get-RunningBuilds

if ($runningBuilds.Count -eq 0) {
    Write-Host "No active builds found. You can start builds with:" -ForegroundColor Yellow
    Write-Host "  docker-compose build <service-name>" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Available services: backend, auth, billing, security, data, ai, webhook, plugins" -ForegroundColor Cyan
    exit 0
}

Write-Host ""
Write-Host "Press Ctrl+C to stop monitoring" -ForegroundColor Yellow
Write-Host ""

# Start monitoring
if ($Service) {
    Watch-ServiceLogs $Service
} else {
    Watch-AllLogs
}
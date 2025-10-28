#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Monitor Docker Compose build processes for ConHub microservices
.DESCRIPTION
    This script monitors all Docker build processes in real-time, displays their status,
    and provides comprehensive logging for deployment to Microsoft Azure Container Apps.
.PARAMETER LogDirectory
    Directory to store build logs (default: ./logs/builds)
.PARAMETER Services
    Comma-separated list of services to monitor (default: all services)
.PARAMETER RefreshInterval
    Refresh interval in seconds (default: 5)
.EXAMPLE
    .\monitor-builds.ps1
    .\monitor-builds.ps1 -Services "backend,auth,billing" -RefreshInterval 3
#>

param(
    [string]$LogDirectory = "./logs/builds",
    [string]$Services = "",
    [int]$RefreshInterval = 5
)

# Color definitions for better output
$Colors = @{
    Success = "Green"
    Warning = "Yellow" 
    Error = "Red"
    Info = "Cyan"
    Header = "Magenta"
    Progress = "Blue"
}

# Service definitions
$AllServices = @(
    "backend", "auth", "billing", "security", "data", 
    "ai", "webhook", "plugins", "nginx", "postgres"
)

# Parse services parameter
$ServicesToMonitor = if ($Services) { 
    $Services -split "," | ForEach-Object { $_.Trim() }
} else { 
    $AllServices 
}

# Create log directory
if (!(Test-Path $LogDirectory)) {
    New-Item -ItemType Directory -Path $LogDirectory -Force | Out-Null
    Write-Host "Created log directory: $LogDirectory" -ForegroundColor $Colors.Info
}

# Function to get timestamp
function Get-Timestamp {
    return Get-Date -Format "yyyy-MM-dd HH:mm:ss"
}

# Function to write log with timestamp
function Write-Log {
    param([string]$Message, [string]$Level = "INFO", [string]$Service = "SYSTEM")
    $timestamp = Get-Timestamp
    $logEntry = "[$timestamp] [$Level] [$Service] $Message"
    
    # Write to console with colors
    $color = switch ($Level) {
        "ERROR" { $Colors.Error }
        "WARN" { $Colors.Warning }
        "SUCCESS" { $Colors.Success }
        default { $Colors.Info }
    }
    Write-Host $logEntry -ForegroundColor $color
    
    # Write to log file
    $logFile = Join-Path $LogDirectory "monitor-$(Get-Date -Format 'yyyy-MM-dd').log"
    Add-Content -Path $logFile -Value $logEntry
}

# Function to check if Docker is running
function Test-DockerRunning {
    try {
        $null = docker info 2>$null
        return $true
    }
    catch {
        return $false
    }
}

# Function to get build status for a service
function Get-BuildStatus {
    param([string]$ServiceName)
    
    try {
        $result = docker-compose ps $ServiceName 2>$null
        if ($LASTEXITCODE -eq 0 -and $result) {
            return "Built"
        }
        
        # Check if build is in progress
        $processes = Get-Process | Where-Object { $_.ProcessName -eq "docker-compose" }
        if ($processes) {
            return "Building"
        }
        
        return "Not Built"
    }
    catch {
        return "Unknown"
    }
}

# Function to start build monitoring for a service
function Start-ServiceBuild {
    param([string]$ServiceName)
    
    $logFile = Join-Path $LogDirectory "$ServiceName-build.log"
    Write-Log "Starting build for service: $ServiceName" "INFO" $ServiceName
    
    try {
        # Start build process and capture output
        $process = Start-Process -FilePath "docker-compose" -ArgumentList "build", $ServiceName -NoNewWindow -PassThru -RedirectStandardOutput $logFile -RedirectStandardError "$logFile.error"
        return $process
    }
    catch {
        Write-Log "Failed to start build for $ServiceName`: $_" "ERROR" $ServiceName
        return $null
    }
}

# Function to display service status table
function Show-ServiceStatus {
    param([hashtable]$ServiceStatuses, [hashtable]$BuildProcesses)
    
    Clear-Host
    Write-Host "=" * 80 -ForegroundColor $Colors.Header
    Write-Host "ConHub Docker Build Monitor - $(Get-Timestamp)" -ForegroundColor $Colors.Header
    Write-Host "=" * 80 -ForegroundColor $Colors.Header
    Write-Host ""
    
    # Table header
    Write-Host ("{0,-15} {1,-12} {2,-10} {3,-15} {4,-20}" -f "Service", "Status", "PID", "Build Time", "Last Update") -ForegroundColor $Colors.Header
    Write-Host ("-" * 80) -ForegroundColor $Colors.Header
    
    foreach ($service in $ServicesToMonitor) {
        $status = $ServiceStatuses[$service] ?? "Unknown"
        $process = $BuildProcesses[$service]
        $pid = if ($process -and !$process.HasExited) { $process.Id } else { "N/A" }
        $buildTime = if ($process -and !$process.HasExited) { 
            $elapsed = (Get-Date) - $process.StartTime
            "{0:mm\:ss}" -f $elapsed
        } else { "N/A" }
        
        $color = switch ($status) {
            "Built" { $Colors.Success }
            "Building" { $Colors.Progress }
            "Failed" { $Colors.Error }
            default { $Colors.Warning }
        }
        
        Write-Host ("{0,-15} {1,-12} {2,-10} {3,-15} {4,-20}" -f $service, $status, $pid, $buildTime, (Get-Timestamp)) -ForegroundColor $color
    }
    
    Write-Host ""
    Write-Host "Logs directory: $LogDirectory" -ForegroundColor $Colors.Info
    Write-Host "Press Ctrl+C to stop monitoring" -ForegroundColor $Colors.Warning
}

# Function to check Azure CLI availability
function Test-AzureCLI {
    try {
        $null = az version 2>$null
        return $true
    }
    catch {
        return $false
    }
}

# Function to generate Azure Container Apps deployment script
function New-AzureDeploymentScript {
    $deployScript = @"
#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Deploy ConHub services to Microsoft Azure Container Apps
.DESCRIPTION
    This script deploys all ConHub microservices to Azure Container Apps Environment
#>

param(
    [Parameter(Mandatory=`$true)]
    [string]`$ResourceGroupName,
    
    [Parameter(Mandatory=`$true)]
    [string]`$ContainerAppEnvironment,
    
    [Parameter(Mandatory=`$true)]
    [string]`$ContainerRegistry,
    
    [string]`$Location = "East US"
)

# Service configurations for Azure Container Apps
`$Services = @{
    "backend" = @{
        "image" = "`$ContainerRegistry/conhub-backend:latest"
        "port" = 8000
        "cpu" = "1.0"
        "memory" = "2Gi"
        "minReplicas" = 1
        "maxReplicas" = 10
    }
    "auth" = @{
        "image" = "`$ContainerRegistry/conhub-auth:latest"
        "port" = 8001
        "cpu" = "0.5"
        "memory" = "1Gi"
        "minReplicas" = 1
        "maxReplicas" = 5
    }
    "billing" = @{
        "image" = "`$ContainerRegistry/conhub-billing:latest"
        "port" = 8002
        "cpu" = "0.5"
        "memory" = "1Gi"
        "minReplicas" = 1
        "maxReplicas" = 3
    }
    "security" = @{
        "image" = "`$ContainerRegistry/conhub-security:latest"
        "port" = 8003
        "cpu" = "0.5"
        "memory" = "1Gi"
        "minReplicas" = 1
        "maxReplicas" = 3
    }
    "data" = @{
        "image" = "`$ContainerRegistry/conhub-data:latest"
        "port" = 8004
        "cpu" = "1.0"
        "memory" = "2Gi"
        "minReplicas" = 1
        "maxReplicas" = 5
    }
    "ai" = @{
        "image" = "`$ContainerRegistry/conhub-ai:latest"
        "port" = 8005
        "cpu" = "2.0"
        "memory" = "4Gi"
        "minReplicas" = 1
        "maxReplicas" = 8
    }
    "webhook" = @{
        "image" = "`$ContainerRegistry/conhub-webhook:latest"
        "port" = 8006
        "cpu" = "0.5"
        "memory" = "1Gi"
        "minReplicas" = 1
        "maxReplicas" = 5
    }
    "plugins" = @{
        "image" = "`$ContainerRegistry/conhub-plugins:latest"
        "port" = 8007
        "cpu" = "1.0"
        "memory" = "2Gi"
        "minReplicas" = 1
        "maxReplicas" = 5
    }
}

Write-Host "Deploying ConHub services to Azure Container Apps..." -ForegroundColor Green

foreach (`$serviceName in `$Services.Keys) {
    `$config = `$Services[`$serviceName]
    
    Write-Host "Deploying `$serviceName..." -ForegroundColor Cyan
    
    az containerapp create \
        --name "conhub-`$serviceName" \
        --resource-group `$ResourceGroupName \
        --environment `$ContainerAppEnvironment \
        --image `$config.image \
        --target-port `$config.port \
        --ingress external \
        --cpu `$config.cpu \
        --memory `$config.memory \
        --min-replicas `$config.minReplicas \
        --max-replicas `$config.maxReplicas \
        --env-vars "PORT=`$(`$config.port)" \
        --query "properties.configuration.ingress.fqdn" \
        --output tsv
        
    if (`$LASTEXITCODE -eq 0) {
        Write-Host "`$serviceName deployed successfully!" -ForegroundColor Green
    } else {
        Write-Host "Failed to deploy `$serviceName" -ForegroundColor Red
    }
}

Write-Host "Deployment completed!" -ForegroundColor Green
"@

    $deployScriptPath = Join-Path $LogDirectory "deploy-to-azure.ps1"
    Set-Content -Path $deployScriptPath -Value $deployScript
    Write-Log "Azure deployment script created: $deployScriptPath" "SUCCESS"
}

# Main monitoring loop
function Start-Monitoring {
    Write-Log "Starting ConHub build monitoring..." "INFO"
    Write-Log "Monitoring services: $($ServicesToMonitor -join ', ')" "INFO"
    
    # Check prerequisites
    if (!(Test-DockerRunning)) {
        Write-Log "Docker is not running. Please start Docker Desktop." "ERROR"
        return
    }
    
    # Generate Azure deployment script
    New-AzureDeploymentScript
    
    if (Test-AzureCLI) {
        Write-Log "Azure CLI detected - deployment script ready" "SUCCESS"
    } else {
        Write-Log "Azure CLI not found - install for deployment capabilities" "WARN"
    }
    
    $serviceStatuses = @{}
    $buildProcesses = @{}
    
    # Initialize service statuses
    foreach ($service in $ServicesToMonitor) {
        $serviceStatuses[$service] = Get-BuildStatus $service
    }
    
    try {
        while ($true) {
            # Update service statuses
            foreach ($service in $ServicesToMonitor) {
                $currentStatus = Get-BuildStatus $service
                $previousStatus = $serviceStatuses[$service]
                
                if ($currentStatus -ne $previousStatus) {
                    Write-Log "Status changed: $service $previousStatus -> $currentStatus" "INFO" $service
                    $serviceStatuses[$service] = $currentStatus
                }
                
                # Check if build process has completed
                $process = $buildProcesses[$service]
                if ($process -and $process.HasExited) {
                    if ($process.ExitCode -eq 0) {
                        $serviceStatuses[$service] = "Built"
                        Write-Log "Build completed successfully" "SUCCESS" $service
                    } else {
                        $serviceStatuses[$service] = "Failed"
                        Write-Log "Build failed with exit code $($process.ExitCode)" "ERROR" $service
                    }
                    $buildProcesses.Remove($service)
                }
            }
            
            # Display current status
            Show-ServiceStatus $serviceStatuses $buildProcesses
            
            # Wait before next update
            Start-Sleep $RefreshInterval
        }
    }
    catch [System.Management.Automation.PipelineStoppedException] {
        Write-Log "Monitoring stopped by user" "INFO"
    }
    finally {
        # Clean up any running processes
        foreach ($process in $buildProcesses.Values) {
            if ($process -and !$process.HasExited) {
                Write-Log "Stopping build process PID: $($process.Id)" "INFO"
                $process.Kill()
            }
        }
    }
}

# Script entry point
Write-Host "ConHub Build Monitor v1.0" -ForegroundColor $Colors.Header
Write-Host "Preparing to monitor Docker builds for Microsoft Azure Container Apps deployment" -ForegroundColor $Colors.Info
Write-Host ""

Start-Monitoring
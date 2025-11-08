# ConHub Build Monitor - Simplified Version
# Real-time monitoring of Docker Compose build processes

param(
    [Parameter(Mandatory=$false)]
    [int]$RefreshInterval = 5
)

# Function to write colored output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

# Function to display header
function Show-Header {
    Clear-Host
    Write-ColorOutput "================================================================" "Cyan"
    Write-ColorOutput "                    ConHub Build Monitor                        " "Cyan"
    Write-ColorOutput "              Real-time Docker Build Status                    " "Cyan"
    Write-ColorOutput "================================================================" "Cyan"
    Write-ColorOutput ""
    Write-ColorOutput "Monitoring ConHub microservices build progress..." "Yellow"
    Write-ColorOutput "Press Ctrl+C to exit" "Gray"
    Write-ColorOutput ""
}

# Function to check Docker Compose processes
function Get-DockerComposeProcesses {
    try {
        $processes = Get-Process | Where-Object { 
            $_.ProcessName -eq "docker-compose" -or 
            $_.ProcessName -eq "docker" 
        } -ErrorAction SilentlyContinue
        return $processes
    }
    catch {
        return @()
    }
}

# Function to get build status for each service
function Get-ServiceBuildStatus {
    $services = @("backend", "auth", "billing", "security", "data", "ai", "webhook", "plugins")
    $status = @{}
    
    foreach ($service in $services) {
        try {
            # Check if image exists
            $imageCheck = docker images --format "{{.Repository}}" | Select-String "conhub-$service" -Quiet
            
            if ($imageCheck) {
                $status[$service] = @{
                    Status = "Completed"
                    Color = "Green"
                }
            }
            else {
                # Check if there's a build process running
                $buildRunning = docker ps -a --format "{{.Names}}" | Select-String "$service" -Quiet
                if ($buildRunning) {
                    $status[$service] = @{
                        Status = "Building"
                        Color = "Yellow"
                    }
                }
                else {
                    $status[$service] = @{
                        Status = "Pending"
                        Color = "Gray"
                    }
                }
            }
        }
        catch {
            $status[$service] = @{
                Status = "Unknown"
                Color = "Red"
            }
        }
    }
    
    return $status
}

# Function to display build status
function Show-BuildStatus {
    param([hashtable]$Status)
    
    Write-ColorOutput "Service Build Status:" "Cyan"
    Write-ColorOutput "--------------------" "Gray"
    
    foreach ($service in $Status.Keys | Sort-Object) {
        $serviceInfo = $Status[$service]
        $serviceName = $service.PadRight(12)
        $statusText = $serviceInfo.Status
        
        Write-Host "  $serviceName : " -NoNewline
        Write-Host $statusText -ForegroundColor $serviceInfo.Color
    }
}

# Function to show running Docker processes
function Show-DockerProcesses {
    Write-ColorOutput "" "White"
    Write-ColorOutput "Active Docker Processes:" "Cyan"
    Write-ColorOutput "------------------------" "Gray"
    
    try {
        $dockerProcesses = docker ps --format "table {{.Names}}\t{{.Status}}\t{{.CreatedAt}}"
        if ($dockerProcesses) {
            $dockerProcesses | ForEach-Object {
                Write-ColorOutput "  $_" "White"
            }
        }
        else {
            Write-ColorOutput "  No active Docker containers" "Gray"
        }
    }
    catch {
        Write-ColorOutput "  Unable to retrieve Docker processes" "Red"
    }
}

# Function to show build logs for a specific service
function Show-BuildLogs {
    param([string]$ServiceName)
    
    Write-ColorOutput "" "White"
    Write-ColorOutput "Recent logs for ${ServiceName}:" "Cyan"
    Write-ColorOutput "-----------------------------" "Gray"
    
    try {
        $logs = docker-compose logs --tail=5 $ServiceName 2>$null
        if ($logs) {
            $logs | ForEach-Object {
                if ($_ -match "error|failed|Error|Failed") {
                    Write-ColorOutput "  $_" "Red"
                }
                elseif ($_ -match "warning|Warning") {
                    Write-ColorOutput "  $_" "Yellow"
                }
                else {
                    Write-ColorOutput "  $_" "White"
                }
            }
        }
        else {
            Write-ColorOutput "  No recent logs available" "Gray"
        }
    }
    catch {
        Write-ColorOutput "  Unable to retrieve logs" "Red"
    }
}

# Function to get overall progress
function Get-OverallProgress {
    param([hashtable]$Status)
    
    $total = $Status.Count
    $completed = ($Status.Values | Where-Object { $_.Status -eq "Completed" }).Count
    $building = ($Status.Values | Where-Object { $_.Status -eq "Building" }).Count
    $pending = ($Status.Values | Where-Object { $_.Status -eq "Pending" }).Count
    
    return @{
        Total = $total
        Completed = $completed
        Building = $building
        Pending = $pending
        PercentComplete = if ($total -gt 0) { [math]::Round(($completed / $total) * 100, 1) } else { 0 }
    }
}

# Function to show progress summary
function Show-ProgressSummary {
    param([hashtable]$Progress)
    
    Write-ColorOutput "" "White"
    Write-ColorOutput "Build Progress Summary:" "Cyan"
    Write-ColorOutput "-----------------------" "Gray"
    Write-ColorOutput "  Total Services: $($Progress.Total)" "White"
    Write-ColorOutput "  Completed: $($Progress.Completed)" "Green"
    Write-ColorOutput "  Building: $($Progress.Building)" "Yellow"
    Write-ColorOutput "  Pending: $($Progress.Pending)" "Gray"
    Write-ColorOutput "  Progress: $($Progress.PercentComplete)%" "Cyan"
}

# Main monitoring loop
Write-ColorOutput "Starting ConHub Build Monitor..." "Green"
Write-ColorOutput ""

try {
    $iteration = 0
    while ($true) {
        $iteration++
        Show-Header
        
        $buildStatus = Get-ServiceBuildStatus
        Show-BuildStatus -Status $buildStatus
        
        Show-DockerProcesses
        
        $progress = Get-OverallProgress -Status $buildStatus
        Show-ProgressSummary -Progress $progress
        
        Write-ColorOutput "" "White"
        $currentTime = Get-Date -Format "HH:mm:ss"
        Write-ColorOutput "Last updated: $currentTime (Iteration: $iteration)" "Gray"
        Write-ColorOutput "Next refresh in $RefreshInterval seconds..." "Gray"
        
        # Check if all builds are complete
        if ($progress.Building -eq 0 -and $progress.Pending -eq 0) {
            Write-ColorOutput "" "White"
            Write-ColorOutput "All builds completed!" "Green"
            break
        }
        
        Start-Sleep -Seconds $RefreshInterval
    }
}
catch {
    Write-ColorOutput "" "White"
    Write-ColorOutput "Monitoring stopped by user." "Yellow"
}

Write-ColorOutput "" "White"
Write-ColorOutput "Build monitoring session ended." "Cyan"
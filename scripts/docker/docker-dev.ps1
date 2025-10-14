


param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("up", "down", "restart", "logs", "build", "clean")]
    [string]$Action = "up",
    
    [Parameter(Mandatory=$false)]
    [string]$Service = "",
    
    [Parameter(Mandatory=$false)]
    [switch]$Build = $false
)

$ErrorActionPreference = "Stop"


$Green = [System.ConsoleColor]::Green
$Yellow = [System.ConsoleColor]::Yellow
$Red = [System.ConsoleColor]::Red
$Blue = [System.ConsoleColor]::Blue

function Write-ColorOutput {
    param([string]$Message, [System.ConsoleColor]$Color = [System.ConsoleColor]::White)
    Write-Host $Message -ForegroundColor $Color
}

function Show-Header {
    Write-ColorOutput "🐳 ConHub Docker Development Environment" $Blue
    Write-ColorOutput "=======================================" $Blue
    Write-Host ""
}

function Check-Prerequisites {
    Write-ColorOutput "🔍 Checking prerequisites..." $Yellow
    
    # Check if Docker is installed and running
    try {
        $dockerVersion = docker --version
        Write-ColorOutput "✅ Docker found: $dockerVersion" $Green
    }
    catch {
        Write-ColorOutput "❌ Docker is not installed or not running" $Red
        Write-ColorOutput "Please install Docker Desktop and ensure it's running" $Red
        exit 1
    }
    
    # Check if docker-compose is available
    try {
        $composeVersion = docker-compose --version
        Write-ColorOutput "✅ Docker Compose found: $composeVersion" $Green
    }
    catch {
        Write-ColorOutput "❌ Docker Compose is not available" $Red
        exit 1
    }
    
    # Check if .env file exists
    if (Test-Path ".env") {
        Write-ColorOutput "✅ Environment file found" $Green
    }
    else {
        Write-ColorOutput "⚠️  .env file not found, using .env.example" $Yellow
        if (Test-Path ".env.example") {
            Copy-Item ".env.example" ".env"
            Write-ColorOutput "✅ Created .env from .env.example" $Green
        }
        else {
            Write-ColorOutput "❌ .env.example not found" $Red
            exit 1
        }
    }
    
    Write-Host ""
}

function Start-Services {
    Write-ColorOutput "🚀 Starting ConHub services..." $Yellow
    
    $composeArgs = @("up", "-d")
    
    if ($Build) {
        $composeArgs += "--build"
    }
    
    if ($Service) {
        $composeArgs += $Service
        Write-ColorOutput "Starting service: $Service" $Blue
    }
    else {
        Write-ColorOutput "Starting all services..." $Blue
    }
    
    try {
        & docker-compose @composeArgs
        Write-ColorOutput "✅ Services started successfully!" $Green
        Show-ServiceStatus
    }
    catch {
        Write-ColorOutput "❌ Failed to start services" $Red
        Write-ColorOutput $_.Exception.Message $Red
        exit 1
    }
}

function Stop-Services {
    Write-ColorOutput "🛑 Stopping ConHub services..." $Yellow
    
    try {
        if ($Service) {
            docker-compose stop $Service
            Write-ColorOutput "✅ Service $Service stopped" $Green
        }
        else {
            docker-compose down
            Write-ColorOutput "✅ All services stopped" $Green
        }
    }
    catch {
        Write-ColorOutput "❌ Failed to stop services" $Red
        Write-ColorOutput $_.Exception.Message $Red
    }
}

function Restart-Services {
    Write-ColorOutput "🔄 Restarting ConHub services..." $Yellow
    Stop-Services
    Start-Sleep -Seconds 2
    Start-Services
}

function Show-Logs {
    Write-ColorOutput "📋 Showing service logs..." $Yellow
    
    if ($Service) {
        docker-compose logs -f $Service
    }
    else {
        docker-compose logs -f
    }
}

function Build-Services {
    Write-ColorOutput "🔨 Building ConHub services..." $Yellow
    
    try {
        if ($Service) {
            docker-compose build $Service
            Write-ColorOutput "✅ Service $Service built successfully" $Green
        }
        else {
            docker-compose build
            Write-ColorOutput "✅ All services built successfully" $Green
        }
    }
    catch {
        Write-ColorOutput "❌ Failed to build services" $Red
        Write-ColorOutput $_.Exception.Message $Red
        exit 1
    }
}

function Clean-Environment {
    Write-ColorOutput "🧹 Cleaning Docker environment..." $Yellow
    
    try {
        # Stop and remove containers
        docker-compose down -v --remove-orphans
        
        # Remove unused images
        docker image prune -f
        
        # Remove unused volumes
        docker volume prune -f
        
        # Remove unused networks
        docker network prune -f
        
        Write-ColorOutput "✅ Environment cleaned successfully" $Green
    }
    catch {
        Write-ColorOutput "❌ Failed to clean environment" $Red
        Write-ColorOutput $_.Exception.Message $Red
    }
}

function Show-ServiceStatus {
    Write-ColorOutput "📊 Service Status:" $Blue
    Write-Host ""
    
    $services = @(
        @{Name="Frontend"; Port="3000"; Path="/"},
        @{Name="Backend"; Port="3001"; Path="/health"},
        @{Name="Lexor"; Port="3002"; Path="/health"},
        @{Name="MCP Service"; Port="3004"; Path="/api/health"},
        @{Name="AI Service"; Port="8001"; Path="/health"},
        @{Name="LangChain"; Port="8003"; Path="/health"}
    )
    
    foreach ($service in $services) {
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:$($service.Port)$($service.Path)" -TimeoutSec 5 -UseBasicParsing
            if ($response.StatusCode -eq 200) {
                Write-ColorOutput "✅ $($service.Name) (http://localhost:$($service.Port))" $Green
            }
            else {
                Write-ColorOutput "⚠️  $($service.Name) (http://localhost:$($service.Port)) - Status: $($response.StatusCode)" $Yellow
            }
        }
        catch {
            Write-ColorOutput "❌ $($service.Name) (http://localhost:$($service.Port)) - Not responding" $Red
        }
    }
    
    Write-Host ""
    Write-ColorOutput "🌐 Access ConHub at: http://localhost:3000" $Blue
    Write-Host ""
}

function Show-Help {
    Write-ColorOutput "ConHub Docker Development Commands:" $Blue
    Write-Host ""
    Write-Host "  up       - Start all services (default)"
    Write-Host "  down     - Stop all services"
    Write-Host "  restart  - Restart all services"
    Write-Host "  logs     - Show service logs"
    Write-Host "  build    - Build services"
    Write-Host "  clean    - Clean Docker environment"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Service <name>  - Target specific service"
    Write-Host "  -Build           - Build images before starting"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\docker-dev.ps1 up -Build"
    Write-Host "  .\docker-dev.ps1 logs -Service backend"
    Write-Host "  .\docker-dev.ps1 restart -Service frontend"
    Write-Host ""
}


Show-Header

if ($Action -eq "help" -or $Action -eq "h") {
    Show-Help
    exit 0
}

Check-Prerequisites

switch ($Action) {
    "up" { Start-Services }
    "down" { Stop-Services }
    "restart" { Restart-Services }
    "logs" { Show-Logs }
    "build" { Build-Services }
    "clean" { Clean-Environment }
    default { 
        Write-ColorOutput "❌ Unknown action: $Action" $Red
        Show-Help
        exit 1
    }
}

Write-ColorOutput "🎉 Operation completed!" $Green

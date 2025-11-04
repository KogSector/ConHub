# Build and Push Docker Images to Docker Hub

param(
    [Parameter(Mandatory=$true)]
    [string]$DockerHubUsername,

    [Parameter(Mandatory=$false)]
    [string]$ImageTag = "latest"
)

# Function to write colored output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

# Function to login to Docker Hub
function Connect-DockerHub {
    Write-ColorOutput "🔐 Logging in to Docker Hub..." "Yellow"
    # This will prompt for your username and password
    docker login
    if ($LASTEXITCODE -eq 0) {
        Write-ColorOutput "✓ Successfully logged in to Docker Hub" "Green"
        return $true
    }
    else {
        Write-ColorOutput "✗ Failed to login to Docker Hub" "Red"
        return $false
    }
}

# Define the services to build
$services = @(
    "frontend", "backend", "auth", "billing", "security", "data", "client", 
    "webhook", "plugins", "indexers", "embedding", "nginx"
)

# Main script
Write-ColorOutput "🚀 Starting Docker image build and push process to Docker Hub" "Cyan"
Write-ColorOutput "============================================================" "Cyan"

if (-not (Connect-DockerHub)) {
    exit 1
}

foreach ($service in $services) {
    $imageName = "conhub-$service"
    $imageFullName = "$DockerHubUsername/$imageName`:$ImageTag"
    $dockerfilePath = "$PSScriptRoot/../../$service/Dockerfile"
    $buildContext = "$PSScriptRoot/../../"

    Write-ColorOutput "Building image for $service..." "Yellow"
    
    docker build -t $imageFullName -f $dockerfilePath $buildContext
    
    if ($LASTEXITCODE -eq 0) {
        Write-ColorOutput "✓ Image for $service built successfully" "Green"
        
        Write-ColorOutput "Pushing image $imageFullName..." "Yellow"
        docker push $imageFullName
        
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput "✓ Image for $service pushed successfully" "Green"
        }
        else {
            Write-ColorOutput "✗ Failed to push image for $service" "Red"
        }
    }
    else {
        Write-ColorOutput "✗ Failed to build image for $service" "Red"
    }
    Write-ColorOutput "" "White"
}

Write-ColorOutput "🎉 All images have been built and pushed successfully to Docker Hub!" "Green"

# Build and Push Docker Images to Azure Container Registry

param(
    [Parameter(Mandatory=$true)]
    [string]$ContainerRegistryName,

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

# Function to login to Azure Container Registry
function Connect-ACR {
    param([string]$RegistryName)
    
    Write-ColorOutput "🔐 Logging in to Azure Container Registry '$RegistryName'..." "Yellow"
    az acr login --name $RegistryName
    if ($LASTEXITCODE -eq 0) {
        Write-ColorOutput "✓ Successfully logged in to ACR" "Green"
        return $true
    }
    else {
        Write-ColorOutput "✗ Failed to login to ACR" "Red"
        return $false
    }
}

# Define the services to build
$services = @(
    "frontend", "backend", "auth", "billing", "security", "data", "client", 
    "webhook", "plugins", "indexers", "embedding", "nginx"
)

# Main script
Write-ColorOutput "🚀 Starting Docker image build and push process" "Cyan"
Write-ColorOutput "===============================================" "Cyan"

if (-not (Connect-ACR -RegistryName $ContainerRegistryName)) {
    exit 1
}

$registryUrl = "$ContainerRegistryName.azurecr.io"

foreach ($service in $services) {
    $imageName = "conhub-$service"
    $imageFullName = "$registryUrl/$imageName`:$ImageTag"
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

Write-ColorOutput "🎉 All images have been built and pushed successfully!" "Green"

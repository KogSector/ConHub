#
# Description: Builds, tags, and pushes a Docker image for a specific microservice in the monorepo.
#
# Usage:
#   .\scripts\services\deploy-service.ps1 -ServiceName auth -Version 1.0.0
#
# Parameters:
#   - ServiceName: (Required) The name of the service directory to build (e.g., 'auth', 'backend').
#   - Version: (Optional) The version tag for the Docker image. Defaults to 'latest'.
#   - ContainerRegistry: (Optional) The name of your container registry.
#

param (
    [Parameter(Mandatory=$true)]
    [string]$ServiceName,

    [Parameter(Mandatory=$false)]
    [string]$Version = "latest",

    [Parameter(Mandatory=$false)]
    [string]$ContainerRegistry = "your-container-registry.azurecr.io" # <-- IMPORTANT: Replace with your Azure Container Registry name
)

# Get the project root directory based on the script's location
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Resolve-Path (Join-Path $ScriptDir "..\..")

$ServicePath = Join-Path $ProjectRoot $ServiceName

# --- 1. Validations ---
Write-Host "Validating service '$ServiceName'..." -ForegroundColor Cyan

if (-not (Test-Path -Path $ServicePath -PathType Container)) {
    Write-Error "Error: Service directory not found at '$ServicePath'."
    exit 1
}

$Dockerfile = Join-Path $ServicePath "Dockerfile"
if (-not (Test-Path -Path $Dockerfile -PathType Leaf)) {
    Write-Error "Error: Dockerfile not found for service '$ServiceName' at '$Dockerfile'."
    exit 1
}

Write-Host "Validation successful." -ForegroundColor Green

# --- 2. Docker Operations ---
$ImageName = "$ContainerRegistry/$ServiceName"
$ImageTag = "$ImageName`:$Version"
$LatestTag = "$ImageName`:latest"

Write-Host "----------------------------------------"
Write-Host "Service: $ServiceName"
Write-Host "Image Tag: $ImageTag"
Write-Host "----------------------------------------"

# Build the Docker image
Write-Host "Building Docker image..." -ForegroundColor Cyan
docker build -t $ImageTag -f $Dockerfile $ServicePath
if ($LASTEXITCODE -ne 0) {
    Write-Error "Docker build failed."
    exit 1
}
Write-Host "Docker build successful." -ForegroundColor Green

# If a specific version is provided, also tag it as 'latest'
if ($Version -ne "latest") {
    Write-Host "Tagging image as 'latest'..." -ForegroundColor Cyan
    docker tag $ImageTag $LatestTag
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Docker tag failed."
        exit 1
    }
}

# Push the Docker image to the registry
Write-Host "Pushing Docker image to $ContainerRegistry..." -ForegroundColor Cyan
Write-Host "(Ensure you are logged in to your container registry: 'az acr login -n your-container-registry')"

docker push $ImageTag
if ($LASTEXITCODE -ne 0) {
    Write-Error "Docker push for tag '$ImageTag' failed."
    exit 1
}

if ($Version -ne "latest") {
    docker push $LatestTag
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Docker push for tag '$LatestTag' failed."
        exit 1
    }
}
Write-Host "Docker push successful." -ForegroundColor Green

# --- 3. Next Steps in Azure Portal ---
Write-Host "----------------------------------------"
Write-Host "Next Steps in the Azure Portal"
Write-Host "----------------------------------------"
Write-Host "Image '$ImageTag' has been pushed successfully to your container registry." -ForegroundColor Green
Write-Host "You can now deploy this new image to your Container App using the Azure Portal:" -ForegroundColor Yellow
Write-Host "1. Navigate to your Container App in the Azure Portal."
Write-Host "2. Go to the 'Revision management' section in the side menu."
Write-Host "3. Click the '+ Create new revision' button."
Write-Host "4. In the 'Image' section, select the new image tag ('$Version') you just pushed."
Write-Host "5. Click the 'Create' button to start the deployment of the new revision."

Write-Host "`nScript finished successfully." -ForegroundColor Green

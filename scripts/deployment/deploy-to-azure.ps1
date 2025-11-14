# Deploy ConHub to Azure Container Apps
# This script deploys all ConHub microservices to Microsoft Azure Container Apps

param(
    [Parameter(Mandatory=$true)]
    [string]$ResourceGroupName,
    
    [Parameter(Mandatory=$true)]
    [string]$ContainerAppEnvironmentName,
    
    [Parameter(Mandatory=$true)]
    [string]$DockerHubUsername,
    
    [Parameter(Mandatory=$false)]
    [string]$Location = "East US",
    
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

# Function to check if Azure CLI is installed
function Test-AzureCLI {
    try {
        $azVersion = az version --output json | ConvertFrom-Json
        Write-ColorOutput "✓ Azure CLI version $($azVersion.'azure-cli') detected" "Green"
        return $true
    }
    catch {
        Write-ColorOutput "✗ Azure CLI not found. Please install Azure CLI first." "Red"
        return $false
    }
}

# Function to login to Azure
function Connect-Azure {
    Write-ColorOutput "🔐 Checking Azure authentication..." "Yellow"
    
    try {
        $account = az account show --output json | ConvertFrom-Json
        Write-ColorOutput "✓ Already logged in as $($account.user.name)" "Green"
        return $true
    }
    catch {
        Write-ColorOutput "🔐 Please login to Azure..." "Yellow"
        az login
        return $?
    }
}

# Function to create resource group if it doesn't exist
function New-ResourceGroupIfNotExists {
    param([string]$Name, [string]$Location)
    
    Write-ColorOutput "📦 Checking resource group '$Name'..." "Yellow"
    
    $rg = az group show --name $Name --output json 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-ColorOutput "✓ Resource group '$Name' already exists" "Green"
    }
    else {
        Write-ColorOutput "📦 Creating resource group '$Name'..." "Yellow"
        az group create --name $Name --location $Location
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput "✓ Resource group '$Name' created successfully" "Green"
        }
        else {
            Write-ColorOutput "✗ Failed to create resource group" "Red"
            exit 1
        }
    }
}

# Function to create Container Apps environment
function New-ContainerAppEnvironment {
    param([string]$Name, [string]$ResourceGroup, [string]$Location)
    
    Write-ColorOutput "🌐 Checking Container Apps environment '$Name'..." "Yellow"
    
    $env = az containerapp env show --name $Name --resource-group $ResourceGroup --output json 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-ColorOutput "✓ Container Apps environment '$Name' already exists" "Green"
    }
    else {
        Write-ColorOutput "🌐 Creating Container Apps environment '$Name'..." "Yellow"
        az containerapp env create --name $Name --resource-group $ResourceGroup --location $Location
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput "✓ Container Apps environment '$Name' created successfully" "Green"
        }
        else {
            Write-ColorOutput "✗ Failed to create Container Apps environment" "Red"
            exit 1
        }
    }
}

# Function to deploy a container app
function Deploy-ContainerApp {
    param(
        [string]$AppName,
        [string]$Image,
        [int]$Port,
        [string]$Environment,
        [string]$ResourceGroup,
        [hashtable]$EnvVars = @{},
        [int]$MinReplicas = 1,
        [int]$MaxReplicas = 3,
        [string]$Memory = "1Gi",
        [string]$CPU = "0.5"
    )
    
    Write-ColorOutput "🚀 Deploying $AppName..." "Yellow"
    
    # Build environment variables string
    $envString = ""
    foreach ($key in $EnvVars.Keys) {
        $envString += "--env-vars $key=$($EnvVars[$key]) "
    }
    
    # Check if app exists
    $existingApp = az containerapp show --name $AppName --resource-group $ResourceGroup --output json 2>$null
    
    if ($LASTEXITCODE -eq 0) {
        # Update existing app
        Write-ColorOutput "🔄 Updating existing container app '$AppName'..." "Yellow"
        $cmd = "az containerapp update --name $AppName --resource-group $ResourceGroup --image $Image $envString"
    }
    else {
        # Create new app
        Write-ColorOutput "🆕 Creating new container app '$AppName'..." "Yellow"
        $cmd = "az containerapp create --name $AppName --resource-group $ResourceGroup --environment $Environment --image $Image --target-port $Port --ingress external --min-replicas $MinReplicas --max-replicas $MaxReplicas --memory $Memory --cpu $CPU $envString"
    }
    
    Invoke-Expression $cmd
    
    if ($LASTEXITCODE -eq 0) {
        Write-ColorOutput "✓ $AppName deployed successfully" "Green"
        
        # Get the app URL
        $appInfo = az containerapp show --name $AppName --resource-group $ResourceGroup --query "properties.configuration.ingress.fqdn" --output tsv
        if ($appInfo) {
            Write-ColorOutput "🌐 $AppName URL: https://$appInfo" "Cyan"
        }
    }
    else {
        Write-ColorOutput "✗ Failed to deploy $AppName" "Red"
    }
}

# Main deployment script
Write-ColorOutput "🚀 Starting ConHub deployment to Azure Container Apps" "Cyan"
Write-ColorOutput "=================================================" "Cyan"

# Check prerequisites
if (-not (Test-AzureCLI)) {
    exit 1
}

if (-not (Connect-Azure)) {
    Write-ColorOutput "✗ Failed to authenticate with Azure" "Red"
    exit 1
}

# Create resources
New-ResourceGroupIfNotExists -Name $ResourceGroupName -Location $Location
New-ContainerAppEnvironment -Name $ContainerAppEnvironmentName -ResourceGroup $ResourceGroupName -Location $Location

# Define services configuration
$services = @{
    "conhub-backend" = @{
        Image = "$DockerHubUsername/conhub-backend:$ImageTag"
        Port = 8000
        EnvVars = @{
            "PORT" = "8000"
            "RUST_LOG" = "info"
        }
        Memory = "2Gi"
        CPU = "1.0"
        MinReplicas = 2
        MaxReplicas = 5
    }
    "conhub-auth" = @{
        Image = "$DockerHubUsername/conhub-auth:$ImageTag"
        Port = 8001
        EnvVars = @{
            "PORT" = "8001"
            "RUST_LOG" = "info"
        }
        Memory = "1Gi"
        CPU = "0.5"
        MinReplicas = 1
        MaxReplicas = 3
    }
    "conhub-billing" = @{
        Image = "$DockerHubUsername/conhub-billing:$ImageTag"
        Port = 8002
        EnvVars = @{
            "PORT" = "8002"
            "RUST_LOG" = "info"
        }
        Memory = "1Gi"
        CPU = "0.5"
        MinReplicas = 1
        MaxReplicas = 3
    }
    "conhub-security" = @{
        Image = "$DockerHubUsername/conhub-security:$ImageTag"
        Port = 8003
        EnvVars = @{
            "PORT" = "8003"
            "RUST_LOG" = "info"
        }
        Memory = "1Gi"
        CPU = "0.5"
        MinReplicas = 1
        MaxReplicas = 3
    }
    "conhub-data" = @{
        Image = "$DockerHubUsername/conhub-data:$ImageTag"
        Port = 8004
        EnvVars = @{
            "PORT" = "8004"
            "RUST_LOG" = "info"
        }
        Memory = "2Gi"
        CPU = "1.0"
        MinReplicas = 1
        MaxReplicas = 4
    }
    "conhub-ai" = @{
        Image = "$DockerHubUsername/conhub-ai:$ImageTag"
        Port = 8005
        EnvVars = @{
            "PORT" = "8005"
            "RUST_LOG" = "info"
        }
        Memory = "4Gi"
        CPU = "2.0"
        MinReplicas = 1
        MaxReplicas = 3
    }
    "conhub-webhook" = @{
        Image = "$DockerHubUsername/conhub-webhook:$ImageTag"
        Port = 8006
        EnvVars = @{
            "PORT" = "8006"
            "RUST_LOG" = "info"
        }
        Memory = "1Gi"
        CPU = "0.5"
        MinReplicas = 1
        MaxReplicas = 3
    }
}

# Deploy each service
Write-ColorOutput "🚀 Deploying ConHub services..." "Cyan"
foreach ($serviceName in $services.Keys) {
    $config = $services[$serviceName]
    Deploy-ContainerApp -AppName $serviceName -Image $config.Image -Port $config.Port -Environment $ContainerAppEnvironmentName -ResourceGroup $ResourceGroupName -EnvVars $config.EnvVars -MinReplicas $config.MinReplicas -MaxReplicas $config.MaxReplicas -Memory $config.Memory -CPU $config.CPU
    Write-ColorOutput "" "White"
}

Write-ColorOutput "🎉 ConHub deployment completed!" "Green"
Write-ColorOutput "=================================================" "Cyan"

# Display summary
Write-ColorOutput "📋 Deployment Summary:" "Cyan"
Write-ColorOutput "Resource Group: $ResourceGroupName" "White"
Write-ColorOutput "Container Apps Environment: $ContainerAppEnvironmentName" "White"
Write-ColorOutput "Location: $Location" "White"
Write-ColorOutput "Image Tag: $ImageTag" "White"
Write-ColorOutput "" "White"
Write-ColorOutput "To view your deployed apps:" "Yellow"
Write-ColorOutput "az containerapp list --resource-group $ResourceGroupName --output table" "Gray"

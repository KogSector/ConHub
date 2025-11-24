#!/usr/bin/env node

const { execSync } = require('child_process');

const colors = {
  cyan: '\x1b[36m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  gray: '\x1b[90m',
  reset: '\x1b[0m',
};

function log(message, color = 'reset') {
  const c = colors[color] || colors.reset;
  process.stdout.write(`${c}${message}${colors.reset}\n`);
}

function run(command, description, options = {}) {
  try {
    if (description) {
      log(description, 'yellow');
    }
    execSync(command, { stdio: 'inherit', ...options });
    return true;
  } catch (err) {
    if (description) {
      log(`✗ ${description}`, 'red');
    }
    return false;
  }
}

function parseArgs() {
  const args = process.argv.slice(2);
  let resourceGroupName = null;
  let envName = null;
  let dockerHubUsername = null;
  let location = 'East US';
  let imageTag = 'latest';

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === '-ResourceGroupName' || arg === '--resource-group' || arg === '-g') {
      resourceGroupName = args[i + 1];
      i += 1;
    } else if (arg === '-ContainerAppEnvironmentName' || arg === '--env' || arg === '-e') {
      envName = args[i + 1];
      i += 1;
    } else if (arg === '-DockerHubUsername' || arg === '--docker-username' || arg === '-u') {
      dockerHubUsername = args[i + 1];
      i += 1;
    } else if (arg === '-Location' || arg === '--location' || arg === '-l') {
      location = args[i + 1];
      i += 1;
    } else if (arg === '-ImageTag' || arg === '--tag' || arg === '-t') {
      imageTag = args[i + 1];
      i += 1;
    }
  }

  if (!resourceGroupName || !envName || !dockerHubUsername) {
    log('Usage: node scripts/deployment/deploy-to-azure.js \
  --resource-group <name> --env <container-app-env> --docker-username <dockerhub-user> [--location <loc>] [--tag <image-tag>]', 'red');
    process.exit(1);
  }

  return { resourceGroupName, envName, dockerHubUsername, location, imageTag };
}

function ensureAzureCli() {
  try {
    const out = execSync('az version --output json', { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] });
    if (out) {
      log('✓ Azure CLI detected', 'green');
      return true;
    }
  } catch (err) {
    // fall through
  }
  log('✗ Azure CLI not found. Please install Azure CLI first.', 'red');
  return false;
}

function ensureAzureLogin() {
  log('🔐 Checking Azure authentication...', 'yellow');
  try {
    execSync('az account show --output json', { stdio: ['ignore', 'ignore', 'ignore'] });
    log('✓ Already logged in to Azure', 'green');
    return true;
  } catch (err) {
    log('🔐 Please login to Azure...', 'yellow');
    try {
      execSync('az login', { stdio: 'inherit' });
      return true;
    } catch (e) {
      log('✗ Failed to authenticate with Azure', 'red');
      return false;
    }
  }
}

function ensureResourceGroup(name, location) {
  log(`📦 Checking resource group '${name}'...`, 'yellow');
  try {
    execSync(`az group show --name "${name}" --output json`, { stdio: ['ignore', 'ignore', 'ignore'] });
    log(`✓ Resource group '${name}' already exists`, 'green');
  } catch (err) {
    if (!run(`az group create --name "${name}" --location "${location}"`, `Creating resource group '${name}'...`)) {
      process.exit(1);
    }
    log(`✓ Resource group '${name}' created successfully`, 'green');
  }
}

function ensureContainerAppEnv(name, resourceGroup, location) {
  log(`🌐 Checking Container Apps environment '${name}'...`, 'yellow');
  try {
    execSync(`az containerapp env show --name "${name}" --resource-group "${resourceGroup}" --output json`, {
      stdio: ['ignore', 'ignore', 'ignore'],
    });
    log(`✓ Container Apps environment '${name}' already exists`, 'green');
  } catch (err) {
    if (!run(
      `az containerapp env create --name "${name}" --resource-group "${resourceGroup}" --location "${location}"`,
      `Creating Container Apps environment '${name}'...`,
    )) {
      process.exit(1);
    }
    log(`✓ Container Apps environment '${name}' created successfully`, 'green');
  }
}

function deployContainerApp({
  appName,
  image,
  port,
  envName,
  resourceGroup,
  envVars = {},
  minReplicas = 1,
  maxReplicas = 3,
  memory = '1Gi',
  cpu = '0.5',
}) {
  log(`🚀 Deploying ${appName}...`, 'yellow');

  const envString = Object.entries(envVars)
    .map(([k, v]) => `${k}=${v}`)
    .join(' ');

  let exists = false;
  try {
    execSync(`az containerapp show --name "${appName}" --resource-group "${resourceGroup}" --output json`, {
      stdio: ['ignore', 'ignore', 'ignore'],
    });
    exists = true;
  } catch (err) {
    exists = false;
  }

  let command;
  if (exists) {
    command = `az containerapp update --name "${appName}" --resource-group "${resourceGroup}" --image "${image}"`;
    if (envString) {
      command += ` --env-vars ${envString}`;
    }
  } else {
    command = `az containerapp create --name "${appName}" --resource-group "${resourceGroup}" --environment "${envName}" --image "${image}" --target-port ${port} --ingress external --min-replicas ${minReplicas} --max-replicas ${maxReplicas} --memory ${memory} --cpu ${cpu}`;
    if (envString) {
      command += ` --env-vars ${envString}`;
    }
  }

  if (!run(command, exists ? `Updating container app '${appName}'...` : `Creating container app '${appName}'...`)) {
    log(`✗ Failed to deploy ${appName}`, 'red');
    return;
  }

  try {
    const fqdn = execSync(
      `az containerapp show --name "${appName}" --resource-group "${resourceGroup}" --query "properties.configuration.ingress.fqdn" --output tsv`,
      { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] },
    )
      .toString()
      .trim();
    if (fqdn) {
      log(`🌐 ${appName} URL: https://${fqdn}`, 'cyan');
    }
  } catch (err) {
    // ignore URL fetch errors
  }

  log(`✓ ${appName} deployed successfully`, 'green');
}

function main() {
  const { resourceGroupName, envName, dockerHubUsername, location, imageTag } = parseArgs();

  log('🚀 Starting ConHub deployment to Azure Container Apps', 'cyan');
  log('=================================================', 'cyan');

  if (!ensureAzureCli()) {
    process.exit(1);
  }

  if (!ensureAzureLogin()) {
    process.exit(1);
  }

  ensureResourceGroup(resourceGroupName, location);
  ensureContainerAppEnv(envName, resourceGroupName, location);

  const services = {
    'conhub-backend': {
      image: `${dockerHubUsername}/conhub-backend:${imageTag}`,
      port: 8000,
      envVars: {
        PORT: '8000',
        RUST_LOG: 'info',
      },
      memory: '2Gi',
      cpu: '1.0',
      minReplicas: 2,
      maxReplicas: 5,
    },
    'conhub-auth': {
      image: `${dockerHubUsername}/conhub-auth:${imageTag}`,
      port: 8001,
      envVars: {
        PORT: '8001',
        RUST_LOG: 'info',
      },
      memory: '1Gi',
      cpu: '0.5',
      minReplicas: 1,
      maxReplicas: 3,
    },
    'conhub-billing': {
      image: `${dockerHubUsername}/conhub-billing:${imageTag}`,
      port: 8002,
      envVars: {
        PORT: '8002',
        RUST_LOG: 'info',
      },
      memory: '1Gi',
      cpu: '0.5',
      minReplicas: 1,
      maxReplicas: 3,
    },
    'conhub-security': {
      image: `${dockerHubUsername}/conhub-security:${imageTag}`,
      port: 8003,
      envVars: {
        PORT: '8003',
        RUST_LOG: 'info',
      },
      memory: '1Gi',
      cpu: '0.5',
      minReplicas: 1,
      maxReplicas: 3,
    },
    'conhub-data': {
      image: `${dockerHubUsername}/conhub-data:${imageTag}`,
      port: 8004,
      envVars: {
        PORT: '8004',
        RUST_LOG: 'info',
      },
      memory: '2Gi',
      cpu: '1.0',
      minReplicas: 1,
      maxReplicas: 4,
    },
    'conhub-ai': {
      image: `${dockerHubUsername}/conhub-ai:${imageTag}`,
      port: 8005,
      envVars: {
        PORT: '8005',
        RUST_LOG: 'info',
      },
      memory: '4Gi',
      cpu: '2.0',
      minReplicas: 1,
      maxReplicas: 3,
    },
    'conhub-webhook': {
      image: `${dockerHubUsername}/conhub-webhook:${imageTag}`,
      port: 8006,
      envVars: {
        PORT: '8006',
        RUST_LOG: 'info',
      },
      memory: '1Gi',
      cpu: '0.5',
      minReplicas: 1,
      maxReplicas: 3,
    },
  };

  log('🚀 Deploying ConHub services...', 'cyan');
  Object.entries(services).forEach(([appName, cfg]) => {
    deployContainerApp({
      appName,
      image: cfg.image,
      port: cfg.port,
      envName,
      resourceGroup: resourceGroupName,
      envVars: cfg.envVars,
      minReplicas: cfg.minReplicas,
      maxReplicas: cfg.maxReplicas,
      memory: cfg.memory,
      cpu: cfg.cpu,
    });
    log('', 'reset');
  });

  log('🎉 ConHub deployment completed!', 'green');
  log('=================================================', 'cyan');

  log('📋 Deployment Summary:', 'cyan');
  log(`Resource Group: ${resourceGroupName}`, 'reset');
  log(`Container Apps Environment: ${envName}`, 'reset');
  log(`Location: ${location}`, 'reset');
  log(`Image Tag: ${imageTag}`, 'reset');
  log('', 'reset');
  log('To view your deployed apps:', 'yellow');
  log('az containerapp list --resource-group <your-resource-group> --output table', 'gray');
}

if (require.main === module) {
  main();
}

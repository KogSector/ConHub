#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const colors = {
  cyan: '\x1b[36m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  reset: '\x1b[0m',
};

function log(message, color = 'reset') {
  const c = colors[color] || colors.reset;
  process.stdout.write(`${c}${message}${colors.reset}\n`);
}

function parseArgs() {
  const args = process.argv.slice(2);
  let serviceName = null;
  let version = 'latest';
  let containerRegistry = 'your-container-registry.azurecr.io';

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === '-ServiceName' || arg === '--service' || arg === '-s') {
      serviceName = args[i + 1];
      i += 1;
    } else if (arg === '-Version' || arg === '--version' || arg === '-v') {
      version = args[i + 1];
      i += 1;
    } else if (arg === '-ContainerRegistry' || arg === '--registry' || arg === '-r') {
      containerRegistry = args[i + 1];
      i += 1;
    }
  }

  if (!serviceName) {
    log('Usage: node scripts/services/deploy-service.js --service <name> [--version <tag>] [--registry <acr>]', 'red');
    process.exit(1);
  }

  return { serviceName, version, containerRegistry };
}

function main() {
  const { serviceName, version, containerRegistry } = parseArgs();

  const scriptDir = __dirname;
  const projectRoot = path.resolve(scriptDir, '..', '..');
  const servicePath = path.join(projectRoot, serviceName);

  log(`Validating service '${serviceName}'...`, 'cyan');

  if (!fs.existsSync(servicePath) || !fs.statSync(servicePath).isDirectory()) {
    log(`Error: Service directory not found at '${servicePath}'.`, 'red');
    process.exit(1);
  }

  const dockerfilePath = path.join(servicePath, 'Dockerfile');
  if (!fs.existsSync(dockerfilePath)) {
    log(`Error: Dockerfile not found for service '${serviceName}' at '${dockerfilePath}'.`, 'red');
    process.exit(1);
  }

  log('Validation successful.', 'green');

  const imageName = `${containerRegistry}/${serviceName}`;
  const imageTag = `${imageName}:${version}`;
  const latestTag = `${imageName}:latest`;

  log('----------------------------------------', 'reset');
  log(`Service: ${serviceName}`, 'reset');
  log(`Image Tag: ${imageTag}`, 'reset');
  log('----------------------------------------', 'reset');

  try {
    log('Building Docker image...', 'cyan');
    execSync(`docker build -t "${imageTag}" -f "${dockerfilePath}" "${servicePath}"`, { stdio: 'inherit' });
    log('Docker build successful.', 'green');
  } catch (err) {
    log('Docker build failed.', 'red');
    process.exit(1);
  }

  if (version !== 'latest') {
    try {
      log("Tagging image as 'latest'...", 'cyan');
      execSync(`docker tag "${imageTag}" "${latestTag}"`, { stdio: 'inherit' });
    } catch (err) {
      log('Docker tag failed.', 'red');
      process.exit(1);
    }
  }

  try {
    log(`Pushing Docker image to ${containerRegistry}...`, 'cyan');
    log("(Ensure you are logged in to your container registry: 'az acr login -n <registry-name>')", 'reset');
    execSync(`docker push "${imageTag}"`, { stdio: 'inherit' });
    if (version !== 'latest') {
      execSync(`docker push "${latestTag}"`, { stdio: 'inherit' });
    }
    log('Docker push successful.', 'green');
  } catch (err) {
    log(`Docker push failed for tag '${imageTag}'.`, 'red');
    process.exit(1);
  }

  log('----------------------------------------', 'reset');
  log('Next Steps in the Azure Portal', 'reset');
  log('----------------------------------------', 'reset');
  log(`Image '${imageTag}' has been pushed successfully to your container registry.`, 'green');
  log('You can now deploy this new image to your Container App using the Azure Portal:', 'yellow');
  log('1. Navigate to your Container App in the Azure Portal.', 'reset');
  log("2. Go to the 'Revision management' section.", 'reset');
  log("3. Click '+ Create new revision'.", 'reset');
  log("4. Select the new image tag you just pushed.", 'reset');
  log("5. Click 'Create' to deploy the new revision.", 'reset');

  log('\nScript finished successfully.', 'green');
}

if (require.main === module) {
  main();
}

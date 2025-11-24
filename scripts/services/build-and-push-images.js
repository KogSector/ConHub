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
  let username = null;
  let imageTag = 'latest';

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === '-DockerHubUsername' || arg === '--username' || arg === '-u') {
      username = args[i + 1];
      i += 1;
    } else if (arg === '-ImageTag' || arg === '--tag' || arg === '-t') {
      imageTag = args[i + 1];
      i += 1;
    }
  }

  if (!username) {
    log('Usage: node scripts/services/build-and-push-images.js --username <dockerhub-username> [--tag <image-tag>]', 'red');
    process.exit(1);
  }

  return { username, imageTag };
}

function dockerLogin() {
  try {
    log('🔐 Logging in to Docker Hub...', 'yellow');
    execSync('docker login', { stdio: 'inherit' });
    log('✓ Successfully logged in to Docker Hub', 'green');
    return true;
  } catch (err) {
    log('✗ Failed to login to Docker Hub', 'red');
    return false;
  }
}

function main() {
  const { username, imageTag } = parseArgs();

  const services = [
    'frontend',
    'backend',
    'auth',
    'billing',
    'security',
    'data',
    'client',
    'webhook',
    'indexers',
    'embedding',
    'nginx',
  ];

  const projectRoot = path.resolve(__dirname, '..', '..');

  log('🚀 Starting Docker image build and push process to Docker Hub', 'cyan');
  log('============================================================', 'cyan');

  if (!dockerLogin()) {
    process.exit(1);
  }

  services.forEach((service) => {
    const imageName = `conhub-${service}`;
    const imageFullName = `${username}/${imageName}:${imageTag}`;
    const dockerfilePath = path.join(projectRoot, service, 'Dockerfile');

    if (!fs.existsSync(dockerfilePath)) {
      log(`✗ Skipping ${service}: Dockerfile not found at ${dockerfilePath}`, 'yellow');
      return;
    }

    log(`Building image for ${service}...`, 'yellow');

    try {
      execSync(`docker build -t "${imageFullName}" -f "${dockerfilePath}" "${projectRoot}"`, {
        stdio: 'inherit',
      });
      log(`✓ Image for ${service} built successfully`, 'green');

      log(`Pushing image ${imageFullName}...`, 'yellow');
      execSync(`docker push "${imageFullName}"`, { stdio: 'inherit' });
      log(`✓ Image for ${service} pushed successfully`, 'green');
    } catch (err) {
      log(`✗ Failed to build or push image for ${service}`, 'red');
    }

    log('', 'reset');
  });

  log('🎉 Image build and push process completed', 'green');
}

if (require.main === module) {
  main();
}

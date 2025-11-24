#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

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

const services = ['auth', 'billing', 'client', 'data', 'security', 'webhook'];

function readKey(filePath, label) {
  if (!fs.existsSync(filePath)) {
    log(`⚠️  ${label} not found at ${filePath}. Run scripts/utils/generate-jwt-keys.js first.`, 'yellow');
    return null;
  }
  log(`✅ Found ${label.toLowerCase()}`, 'green');
  return fs.readFileSync(filePath, 'utf8');
}

function initializeServiceEnv(projectRoot, serviceName, privateKey, publicKey) {
  const examplePath = path.join(projectRoot, serviceName, '.env.example');
  const envPath = path.join(projectRoot, serviceName, '.env');

  if (!fs.existsSync(examplePath)) {
    log(`⚠️  ${serviceName}/.env.example not found, skipping...`, 'yellow');
    return;
  }

  fs.copyFileSync(examplePath, envPath);
  log(`📝 Created ${serviceName}/.env`, 'cyan');

  let content = fs.readFileSync(envPath, 'utf8');

  const hasPrivate = Boolean(privateKey);
  const hasPublic = Boolean(publicKey);

  const privatePattern = /JWT_PRIVATE_KEY="[^]*?"/m;
  const publicPattern = /JWT_PUBLIC_KEY="[^]*?"/m;

  if (hasPrivate && serviceName === 'auth') {
    content = content.replace(privatePattern, `JWT_PRIVATE_KEY="${privateKey.trim()}"`);
    if (hasPublic) {
      content = content.replace(publicPattern, `JWT_PUBLIC_KEY="${publicKey.trim()}"`);
    }
    log('  ✅ Populated JWT keys (private + public)', 'green');
  } else if (hasPublic) {
    content = content.replace(publicPattern, `JWT_PUBLIC_KEY="${publicKey.trim()}"`);
    log('  ✅ Populated JWT public key', 'green');
  }

  fs.writeFileSync(envPath, content);
}

function main() {
  log('🔧 Setting up environment files for all microservices...', 'cyan');
  log('', 'reset');

  const projectRoot = path.resolve(__dirname, '..', '..');
  const keysDir = path.join(projectRoot, 'keys');

  const privateKeyPath = path.join(keysDir, 'private_key.pem');
  const publicKeyPath = path.join(keysDir, 'public_key.pem');

  const privateKey = readKey(privateKeyPath, 'Private key');
  const publicKey = readKey(publicKeyPath, 'Public key');

  log('', 'reset');

  services.forEach((service) => {
    initializeServiceEnv(projectRoot, service, privateKey, publicKey);
  });

  // Frontend env
  const frontendExample = path.join(projectRoot, 'frontend', '.env.example');
  const frontendEnv = path.join(projectRoot, 'frontend', '.env.local');
  if (fs.existsSync(frontendExample)) {
    fs.copyFileSync(frontendExample, frontendEnv);
    log('📝 Created frontend/.env.local', 'cyan');
  }

  log('', 'reset');
  log('🎉 Environment setup complete!', 'green');
  log('', 'reset');
  log('Next steps:', 'cyan');
  log('1. Review and update the .env files with your actual values', 'reset');
  log('2. Make sure PostgreSQL is running on localhost:5432', 'reset');
  log('3. Make sure Redis is running on localhost:6379', 'reset');
  log('4. Make sure Qdrant is running on localhost:6333 (for AI/Data services)', 'reset');
  log('5. Run your dev/start scripts to start all services', 'reset');
  log('', 'reset');
  log('⚠️  IMPORTANT: Never commit .env files to git!', 'yellow');
}

if (require.main === module) {
  main();
}

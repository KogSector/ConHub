#!/usr/bin/env node

const { execSync } = require('child_process');
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

function main() {
  log('🔑 Generating JWT RSA Key Pair...', 'cyan');

  const projectRoot = path.resolve(__dirname, '..', '..');
  const keysDir = path.join(projectRoot, 'keys');

  if (!fs.existsSync(keysDir)) {
    fs.mkdirSync(keysDir, { recursive: true });
    log('✅ Created keys directory', 'green');
  }

  // Check if OpenSSL is available
  try {
    execSync('openssl version', { stdio: 'ignore' });
  } catch (err) {
    log('❌ OpenSSL not found. Please install OpenSSL first.', 'red');
    log('', 'reset');
    log('Options to install OpenSSL:', 'yellow');
    log('1. macOS (Homebrew):   brew install openssl', 'reset');
    log('2. Linux (apt):        sudo apt-get install openssl', 'reset');
    log('3. Windows (choco):    choco install openssl', 'reset');
    log('', 'reset');
    log('After generating keys, save them as:', 'yellow');
    log('  - keys/private_key.pem (Private Key)', 'reset');
    log('  - keys/public_key.pem (Public Key)', 'reset');
    process.exit(1);
  }

  const privateKeyPath = path.join(keysDir, 'private_key.pem');
  const publicKeyPath = path.join(keysDir, 'public_key.pem');

  // Generate private key
  try {
    log('Generating private key...', 'yellow');
    execSync(`openssl genrsa -out "${privateKeyPath}" 2048`, { stdio: 'ignore' });
    log(`✅ Private key generated: ${path.relative(projectRoot, privateKeyPath)}`, 'green');
  } catch (err) {
    log('❌ Failed to generate private key', 'red');
    process.exit(1);
  }

  // Generate public key
  try {
    log('Generating public key...', 'yellow');
    execSync(`openssl rsa -in "${privateKeyPath}" -pubout -out "${publicKeyPath}"`, { stdio: 'ignore' });
    log(`✅ Public key generated: ${path.relative(projectRoot, publicKeyPath)}`, 'green');
  } catch (err) {
    log('❌ Failed to generate public key', 'red');
    process.exit(1);
  }

  log('', 'reset');
  log('🎉 JWT RSA Key Pair generated successfully!', 'green');
  log('', 'reset');
  log('Next steps:', 'cyan');
  log('1. Copy .env.example to .env in each microservice folder', 'reset');
  log('2. Run scripts/setup/setup-env.js to populate .env files with the keys', 'reset');
  log('', 'reset');
  log('⚠️  IMPORTANT: Keep these keys secure and never commit them to git!', 'yellow');
}

if (require.main === module) {
  main();
}

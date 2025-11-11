#!/usr/bin/env node

/**
 * Generate JWT RSA Key Pair using Node.js crypto
 * No external dependencies required
 */

const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

const colors = {
  cyan: '\x1b[36m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  magenta: '\x1b[35m',
  reset: '\x1b[0m',
  bright: '\x1b[1m'
};

function log(message, color = colors.reset) {
  console.log(`${color}${message}${colors.reset}`);
}

function generateJwtKeys() {
  log(`\n${colors.bright}${colors.magenta}🔑 Generating JWT RSA Key Pair...${colors.reset}\n`);
  
  const projectRoot = path.resolve(__dirname, '..', '..');
  const keysDir = path.join(projectRoot, 'keys');
  
  // Create keys directory if it doesn't exist
  if (!fs.existsSync(keysDir)) {
    fs.mkdirSync(keysDir, { recursive: true });
    log('✅ Created keys directory', colors.green);
  }
  
  const privateKeyPath = path.join(keysDir, 'private_key.pem');
  const publicKeyPath = path.join(keysDir, 'public_key.pem');
  
  // Check if keys already exist
  if (fs.existsSync(privateKeyPath) && fs.existsSync(publicKeyPath)) {
    log('⚠️  JWT keys already exist.', colors.yellow);
    log('   Delete the keys directory to regenerate them.', colors.yellow);
    return {
      privateKey: fs.readFileSync(privateKeyPath, 'utf8'),
      publicKey: fs.readFileSync(publicKeyPath, 'utf8')
    };
  }
  
  try {
    // Generate RSA key pair (2048 bits)
    log('Generating RSA key pair (2048 bits)...', colors.cyan);
    
    const { privateKey, publicKey } = crypto.generateKeyPairSync('rsa', {
      modulusLength: 2048,
      publicKeyEncoding: {
        type: 'spki',
        format: 'pem'
      },
      privateKeyEncoding: {
        type: 'pkcs1',
        format: 'pem'
      }
    });
    
    // Write keys to files
    fs.writeFileSync(privateKeyPath, privateKey, 'utf8');
    fs.writeFileSync(publicKeyPath, publicKey, 'utf8');
    
    log(`✅ Private key generated: ${path.relative(projectRoot, privateKeyPath)}`, colors.green);
    log(`✅ Public key generated: ${path.relative(projectRoot, publicKeyPath)}`, colors.green);
    
    return { privateKey, publicKey };
    
  } catch (error) {
    log(`❌ Failed to generate keys: ${error.message}`, colors.red);
    throw error;
  }
}

if (require.main === module) {
  try {
    generateJwtKeys();
    log(`\n${colors.bright}${colors.green}🎉 JWT RSA Key Pair generated successfully!${colors.reset}\n`);
  } catch (error) {
    log(`\n❌ Failed: ${error.message}`, colors.red);
    process.exit(1);
  }
}

module.exports = { generateJwtKeys };

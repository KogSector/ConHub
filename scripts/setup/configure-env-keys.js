#!/usr/bin/env node

/**
 * Configure .env with JWT key file paths instead of inline keys
 * This avoids multiline formatting issues in .env files
 */

const fs = require('fs');
const path = require('path');

const colors = {
  cyan: '\x1b[36m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  reset: '\x1b[0m',
  bright: '\x1b[1m'
};

function log(message, color = colors.reset) {
  console.log(`${color}${message}${colors.reset}`);
}

function configureEnvWithKeyPaths() {
  const projectRoot = path.resolve(__dirname, '..', '..');
  const envPath = path.join(projectRoot, '.env');
  
  log(`\n${colors.bright}Configuring .env with JWT key file paths...${colors.reset}\n`, colors.cyan);
  
  if (!fs.existsSync(envPath)) {
    log('❌ .env file not found', colors.red);
    return false;
  }
  
  // Check if keys exist
  const privateKeyPath = path.join(projectRoot, 'keys', 'private_key.pem');
  const publicKeyPath = path.join(projectRoot, 'keys', 'public_key.pem');
  
  if (!fs.existsSync(privateKeyPath) || !fs.existsSync(publicKeyPath)) {
    log('❌ JWT keys not found. Run the setup script first.', colors.red);
    return false;
  }
  
  let content = fs.readFileSync(envPath, 'utf8');
  const lines = content.split('\n');
  const newLines = [];
  let inJwtSection = false;
  let foundPrivateKey = false;
  let foundPublicKey = false;
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    
    // Detect JWT section
    if (line.includes('# === JWT RS256 Authentication Configuration ===')) {
      inJwtSection = true;
    } else if (inJwtSection && line.startsWith('# ===')) {
      inJwtSection = false;
    }
    
    // Skip old JWT_PRIVATE_KEY and JWT_PUBLIC_KEY lines
    if (line.startsWith('JWT_PRIVATE_KEY=') || line.startsWith('JWT_PUBLIC_KEY=')) {
      if (line.startsWith('JWT_PRIVATE_KEY=')) {
        foundPrivateKey = true;
      } else {
        foundPublicKey = true;
      }
      continue;
    }
    
    // Add key paths after the key generation comment
    if (inJwtSection && line.includes('# Generate RSA key pair using:')) {
      newLines.push(line);
      // Skip the next few comment lines about key generation
      while (i + 1 < lines.length && (lines[i + 1].startsWith('#') || lines[i + 1].trim() === '')) {
        i++;
        newLines.push(lines[i]);
        if (lines[i].includes('openssl rsa -in private_key.pem')) {
          break;
        }
      }
      
      // Add the key file path configuration
      newLines.push('');
      newLines.push('# Path to RSA keys (recommended - avoids multiline .env issues)');
      newLines.push(`JWT_PRIVATE_KEY_PATH=${path.join(projectRoot, 'keys', 'private_key.pem').replace(/\\/g, '/')}`);
      newLines.push(`JWT_PUBLIC_KEY_PATH=${path.join(projectRoot, 'keys', 'public_key.pem').replace(/\\/g, '/')}`);
      newLines.push('');
      newLines.push('# OR use inline keys (not recommended for Windows):');
      newLines.push('# JWT_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----...');
      newLines.push('# JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----...');
      foundPrivateKey = true;
      foundPublicKey = true;
      continue;
    }
    
    newLines.push(line);
  }
  
  // Write back
  fs.writeFileSync(envPath, newLines.join('\n'), 'utf8');
  
  if (foundPrivateKey && foundPublicKey) {
    log('✅ Configured JWT_PRIVATE_KEY_PATH and JWT_PUBLIC_KEY_PATH', colors.green);
  }
  
  return true;
}

if (require.main === module) {
  try {
    if (configureEnvWithKeyPaths()) {
      log(`\n${colors.bright}${colors.green}✅ Configuration complete!${colors.reset}\n`);
      log('The services will now read JWT keys from file paths instead of inline values.', colors.cyan);
      log('This avoids multiline formatting issues in .env files.\n');
    } else {
      process.exit(1);
    }
  } catch (error) {
    log(`\n❌ Failed: ${error.message}`, colors.red);
    console.error(error);
    process.exit(1);
  }
}

module.exports = { configureEnvWithKeyPaths };

#!/usr/bin/env node

/**
 * Verify ConHub Configuration
 * Checks if all necessary configuration is in place
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

function checkFile(filePath, description) {
  if (fs.existsSync(filePath)) {
    log(`✅ ${description}`, colors.green);
    return true;
  } else {
    log(`❌ ${description} - NOT FOUND`, colors.red);
    return false;
  }
}

function checkEnvVariable(envPath, varName) {
  if (!fs.existsSync(envPath)) {
    return false;
  }
  
  const content = fs.readFileSync(envPath, 'utf8');
  
  // Check for variable with actual value (not empty, not placeholder)
  const regex = new RegExp(`^${varName}=(.+)$`, 'm');
  const match = content.match(regex);
  
  if (match && match[1] && 
      !match[1].includes('YOUR_') && 
      !match[1].includes('your-') &&
      !match[1].includes('your_') &&
      match[1].trim() !== '') {
    log(`  ✅ ${varName} is configured`, colors.green);
    return true;
  } else {
    log(`  ⚠️  ${varName} is not configured or uses placeholder`, colors.yellow);
    return false;
  }
}

function verifyConfiguration() {
  log(`\n${colors.bright}${colors.cyan}🔍 Verifying ConHub Configuration${colors.reset}\n`);
  
  const projectRoot = path.resolve(__dirname, '..', '..');
  let allGood = true;
  
  // Check JWT Keys
  log(`${colors.bright}1. JWT Keys${colors.reset}`, colors.cyan);
  const privateKeyExists = checkFile(
    path.join(projectRoot, 'keys', 'private_key.pem'),
    'Private Key (keys/private_key.pem)'
  );
  const publicKeyExists = checkFile(
    path.join(projectRoot, 'keys', 'public_key.pem'),
    'Public Key (keys/public_key.pem)'
  );
  
  if (!privateKeyExists || !publicKeyExists) {
    log('  ⚠️  Run: node scripts/setup/generate-jwt-keys.js', colors.yellow);
    allGood = false;
  }
  
  // Check .env file
  log(`\n${colors.bright}2. Environment Configuration${colors.reset}`, colors.cyan);
  const envPath = path.join(projectRoot, '.env');
  const envExists = checkFile(envPath, 'Root .env file');
  
  if (!envExists) {
    log('  ⚠️  Run: node scripts/setup/setup-neondb.js', colors.yellow);
    allGood = false;
  } else {
    // Check critical environment variables
    log('\n  Checking environment variables:');
    
    const hasNeonDb = checkEnvVariable(envPath, 'DATABASE_URL_NEON');
    const hasJwtPrivatePath = checkEnvVariable(envPath, 'JWT_PRIVATE_KEY_PATH');
    const hasJwtPublicPath = checkEnvVariable(envPath, 'JWT_PUBLIC_KEY_PATH');
    
    if (!hasNeonDb) {
      log('  ℹ️  DATABASE_URL_NEON not set - will use local PostgreSQL', colors.cyan);
    }
    
    if (!hasJwtPrivatePath || !hasJwtPublicPath) {
      log('  ⚠️  Run: node scripts/setup/configure-env-keys.js', colors.yellow);
      allGood = false;
    }
  }
  
  // Check feature toggles
  log(`\n${colors.bright}3. Feature Toggles${colors.reset}`, colors.cyan);
  const togglesPath = path.join(projectRoot, 'feature-toggles.json');
  const togglesExist = checkFile(togglesPath, 'feature-toggles.json');
  
  if (togglesExist) {
    const toggles = JSON.parse(fs.readFileSync(togglesPath, 'utf8'));
    if (toggles.Auth === true) {
      log('  ✅ Auth feature is enabled', colors.green);
    } else {
      log('  ⚠️  Auth feature is disabled - enable it in feature-toggles.json', colors.yellow);
      allGood = false;
    }
    
    log(`  ℹ️  Heavy features: ${toggles.Heavy ? 'enabled' : 'disabled'}`, colors.cyan);
    log(`  ℹ️  Docker mode: ${toggles.Docker ? 'enabled' : 'disabled'}`, colors.cyan);
  }
  
  // Check package dependencies
  log(`\n${colors.bright}4. Dependencies${colors.reset}`, colors.cyan);
  checkFile(path.join(projectRoot, 'node_modules'), 'Node modules installed');
  checkFile(path.join(projectRoot, 'frontend', 'node_modules'), 'Frontend dependencies');
  
  // Summary
  log(`\n${colors.bright}${'='.repeat(50)}${colors.reset}`);
  
  if (allGood) {
    log(`\n${colors.bright}${colors.green}✅ Configuration looks good!${colors.reset}\n`);
    log('Next steps:', colors.cyan);
    log('1. Make sure Redis is running (required for Auth)');
    log('2. Make sure Qdrant is running (required for Data service)');
    log('3. Run: npm start');
    log('');
  } else {
    log(`\n${colors.bright}${colors.yellow}⚠️  Configuration issues found${colors.reset}\n`);
    log('Please address the issues above and run this script again.', colors.yellow);
    log('');
    log('Quick fix: Run the automated setup:', colors.cyan);
    log('  node scripts/setup/setup-neondb.js', colors.bright);
    log('');
  }
  
  return allGood;
}

if (require.main === module) {
  const isValid = verifyConfiguration();
  process.exit(isValid ? 0 : 1);
}

module.exports = { verifyConfiguration };

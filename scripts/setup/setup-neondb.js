#!/usr/bin/env node

/**
 * ConHub NeonDB Setup Script
 * This script generates JWT keys, configures NeonDB, and updates environment files
 */

const fs = require('fs');
const path = require('path');
const readline = require('readline');
const { generateJwtKeys } = require('./generate-jwt-keys');

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

async function promptUser(question) {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
  });
  
  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      rl.close();
      resolve(answer);
    });
  });
}

function escapeForEnv(str) {
  // Escape newlines for .env file
  return str.replace(/\n/g, '\\n');
}

function updateRootEnvFile(neonDbUrl, privateKey, publicKey) {
  const projectRoot = path.resolve(__dirname, '..', '..');
  const envPath = path.join(projectRoot, '.env');
  const envExamplePath = path.join(projectRoot, '.env.example');
  
  log('\n📝 Updating root .env file...', colors.cyan);
  
  // Read existing .env or create from example
  if (!fs.existsSync(envPath)) {
    if (fs.existsSync(envExamplePath)) {
      fs.copyFileSync(envExamplePath, envPath);
      log('✅ Created .env from .env.example', colors.green);
    } else {
      log('❌ No .env.example found', colors.red);
      return false;
    }
  }
  
  // Read .env file
  let content = fs.readFileSync(envPath, 'utf8');
  
  // Update JWT keys if provided
  if (privateKey) {
    const privateKeyEscaped = escapeForEnv(privateKey.trim());
    
    // Try to find and replace existing JWT_PRIVATE_KEY
    if (content.includes('JWT_PRIVATE_KEY=')) {
      // Remove any existing JWT_PRIVATE_KEY line(s)
      content = content.replace(/JWT_PRIVATE_KEY="[^"]*"/g, '');
      content = content.replace(/JWT_PRIVATE_KEY=[^\n]*/g, '');
    }
    
    // Add JWT_PRIVATE_KEY in the JWT section
    const jwtSectionMatch = content.match(/(# === JWT RS256 Authentication Configuration ===[\s\S]*?)(# Private key)/);
    if (jwtSectionMatch) {
      content = content.replace(
        /(# Private key \(used by auth service to sign JWTs\)\s*\n)/,
        `$1JWT_PRIVATE_KEY="${privateKeyEscaped}"\n\n`
      );
      log('✅ Updated JWT_PRIVATE_KEY', colors.green);
    }
  }
  
  if (publicKey) {
    const publicKeyEscaped = escapeForEnv(publicKey.trim());
    
    // Try to find and replace existing JWT_PUBLIC_KEY
    if (content.includes('JWT_PUBLIC_KEY=')) {
      content = content.replace(/JWT_PUBLIC_KEY="[^"]*"/g, '');
      content = content.replace(/JWT_PUBLIC_KEY=[^\n]*/g, '');
    }
    
    // Add JWT_PUBLIC_KEY in the JWT section
    const jwtPublicMatch = content.match(/(# Public key \(used by other services to verify JWTs\)\s*\n)/);
    if (jwtPublicMatch) {
      content = content.replace(
        /(# Public key \(used by other services to verify JWTs\)\s*\n)/,
        `$1JWT_PUBLIC_KEY="${publicKeyEscaped}"\n\n`
      );
      log('✅ Updated JWT_PUBLIC_KEY', colors.green);
    }
  }
  
  // Update NeonDB URL if provided
  if (neonDbUrl && neonDbUrl.trim().length > 0) {
    // Ensure sslmode=require for Neon
    let finalNeonUrl = neonDbUrl.trim();
    if (!finalNeonUrl.includes('sslmode=require')) {
      finalNeonUrl += finalNeonUrl.includes('?') ? '&sslmode=require' : '?sslmode=require';
    }
    
    // Replace DATABASE_URL_NEON
    content = content.replace(
      /DATABASE_URL_NEON=.*/,
      `DATABASE_URL_NEON=${finalNeonUrl}`
    );
    log('✅ Updated DATABASE_URL_NEON', colors.green);
  }
  
  // Write back to file
  fs.writeFileSync(envPath, content, 'utf8');
  
  return true;
}

function sortEnvVariables(envPath) {
  if (!fs.existsSync(envPath)) {
    return;
  }
  
  log(`\n📋 Organizing environment variables...`, colors.cyan);
  
  const content = fs.readFileSync(envPath, 'utf8');
  const lines = content.split('\n');
  
  // Keep the structure but ensure sections are properly organized
  // For now, we'll just validate that all required sections exist
  const requiredSections = [
    'Environment Mode',
    'Application URLs',
    'Microservices URLs',
    'JWT RS256 Authentication Configuration',
    'Database Configuration'
  ];
  
  const foundSections = [];
  for (const section of requiredSections) {
    if (content.includes(`# === ${section} ===`)) {
      foundSections.push(section);
    }
  }
  
  log(`✅ Found ${foundSections.length}/${requiredSections.length} configuration sections`, colors.green);
}

function updateFeatureToggles() {
  const projectRoot = path.resolve(__dirname, '..', '..');
  const togglesPath = path.join(projectRoot, 'feature-toggles.json');
  
  if (fs.existsSync(togglesPath)) {
    const toggles = JSON.parse(fs.readFileSync(togglesPath, 'utf8'));
    
    if (toggles.Auth !== true) {
      toggles.Auth = true;
      fs.writeFileSync(togglesPath, JSON.stringify(toggles, null, 2) + '\n', 'utf8');
      log('\n✅ Enabled Auth feature toggle', colors.green);
    } else {
      log('\n✅ Auth feature already enabled', colors.green);
    }
  }
}

async function main() {
  log(`${colors.bright}${colors.magenta}🚀 ConHub NeonDB Setup Script${colors.reset}\n`);
  log('This script will configure your ConHub installation with NeonDB\n');
  
  try {
    // Step 1: Generate JWT keys
    log(`${colors.bright}Step 1: Generate JWT Keys${colors.reset}`, colors.cyan);
    const { privateKey, publicKey } = generateJwtKeys();
    
    // Step 2: Get NeonDB URL
    log(`\n${colors.bright}Step 2: Configure NeonDB${colors.reset}`, colors.cyan);
    log('Enter your NeonDB connection string (or press Enter to skip):', colors.yellow);
    log('Example: postgresql://user:password@ep-xxxx.region.neon.tech/dbname', colors.reset);
    const neonDbUrl = await promptUser('> ');
    
    // Step 3: Update root .env file
    log(`\n${colors.bright}Step 3: Update Environment Configuration${colors.reset}`, colors.cyan);
    if (!updateRootEnvFile(neonDbUrl, privateKey, publicKey)) {
      log('\n❌ Failed to update environment file', colors.red);
      process.exit(1);
    }
    
    // Step 4: Sort/organize environment variables
    const projectRoot = path.resolve(__dirname, '..', '..');
    const envPath = path.join(projectRoot, '.env');
    sortEnvVariables(envPath);
    
    // Step 5: Update feature toggles
    log(`\n${colors.bright}Step 4: Configure Feature Toggles${colors.reset}`, colors.cyan);
    updateFeatureToggles();
    
    // Summary
    log(`\n${colors.bright}${colors.green}🎉 Setup Complete!${colors.reset}\n`);
    log('Configuration Summary:', colors.cyan);
    log('✅ JWT keys generated and configured', colors.green);
    
    if (neonDbUrl && neonDbUrl.trim().length > 0) {
      log('✅ NeonDB connection configured', colors.green);
    } else {
      log('⚠️  NeonDB URL not provided - using local PostgreSQL', colors.yellow);
    }
    
    log('✅ Environment variables organized', colors.green);
    log('✅ Auth feature enabled', colors.green);
    
    log('\nNext Steps:', colors.cyan);
    log('1. Review your .env file and update any other necessary values');
    log('2. Ensure Redis is running (required for Auth service)');
    log('3. Ensure Qdrant is running (required for Data service)');
    log('4. Run "npm start" to start all services');
    
    log('\n⚠️  IMPORTANT: Keep your .env file and keys secure!', colors.yellow);
    log('   Never commit .env files or keys to version control', colors.yellow);
    
  } catch (error) {
    log(`\n❌ Setup failed: ${error.message}`, colors.red);
    console.error(error);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}

module.exports = { main };

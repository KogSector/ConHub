#!/usr/bin/env node

/**
 * Fix .env configuration
 * Properly adds/updates JWT key paths and NeonDB URL
 */

const fs = require('fs');
const path = require('path');
const readline = require('readline');

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

function updateOrAddEnvVar(content, varName, value) {
  const lines = content.split('\n');
  let found = false;
  let result = [];
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    
    // Check if this line is the variable we're looking for
    if (line.startsWith(`${varName}=`)) {
      // Replace it
      result.push(`${varName}=${value}`);
      found = true;
    } else {
      result.push(line);
    }
  }
  
  // If not found, add it at the end of the appropriate section
  if (!found) {
    // Find the appropriate section to add the variable
    result = [];
    let inSection = false;
    let sectionEnd = -1;
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      
      // Detect sections
      if (varName.includes('JWT') && line.includes('# === JWT RS256')) {
        inSection = true;
      } else if (varName.includes('DATABASE_URL_NEON') && line.includes('# === Database Configuration ===')) {
        inSection = true;
      }
      
      // Find the end of the section
      if (inSection && line.startsWith('# ===') && !line.includes('JWT') && !line.includes('Database')) {
        sectionEnd = i;
        inSection = false;
      }
      
      result.push(line);
      
      // Insert the variable at the end of the section
      if (sectionEnd === i) {
        result.splice(i, 0, `${varName}=${value}`);
        result.splice(i, 0, '');
        found = true;
        break;
      }
    }
    
    // If still not found, just add at the end
    if (!found) {
      result.push('');
      result.push(`${varName}=${value}`);
    }
  }
  
  return result.join('\n');
}

async function fixEnvConfiguration() {
  log(`\n${colors.bright}${colors.cyan}🔧 Fixing .env Configuration${colors.reset}\n`);
  
  const projectRoot = path.resolve(__dirname, '..', '..');
  const envPath = path.join(projectRoot, '.env');
  const envExamplePath = path.join(projectRoot, '.env.example');
  
  // Ensure .env exists
  if (!fs.existsSync(envPath)) {
    if (fs.existsSync(envExamplePath)) {
      fs.copyFileSync(envExamplePath, envPath);
      log('✅ Created .env from .env.example', colors.green);
    } else {
      log('❌ No .env or .env.example found', colors.red);
      return false;
    }
  }
  
  let content = fs.readFileSync(envPath, 'utf8');
  
  // 1. Configure JWT key paths
  log('Configuring JWT keys...', colors.cyan);
  
  const privateKeyPath = path.join(projectRoot, 'keys', 'private_key.pem').replace(/\\/g, '/');
  const publicKeyPath = path.join(projectRoot, 'keys', 'public_key.pem').replace(/\\/g, '/');
  
  if (fs.existsSync(privateKeyPath.replace(/\//g, '\\'))) {
    content = updateOrAddEnvVar(content, 'JWT_PRIVATE_KEY_PATH', privateKeyPath);
    log(`  ✅ Set JWT_PRIVATE_KEY_PATH=${privateKeyPath}`, colors.green);
  } else {
    log('  ⚠️  Private key not found - run generate-jwt-keys.js first', colors.yellow);
  }
  
  if (fs.existsSync(publicKeyPath.replace(/\//g, '\\'))) {
    content = updateOrAddEnvVar(content, 'JWT_PUBLIC_KEY_PATH', publicKeyPath);
    log(`  ✅ Set JWT_PUBLIC_KEY_PATH=${publicKeyPath}`, colors.green);
  } else {
    log('  ⚠️  Public key not found - run generate-jwt-keys.js first', colors.yellow);
  }
  
  // 2. Configure NeonDB
  log('\nConfiguring NeonDB...', colors.cyan);
  
  // Check if DATABASE_URL_NEON is already set with a real value
  const neonMatch = content.match(/DATABASE_URL_NEON=(.+)/);
  if (neonMatch && neonMatch[1] && 
      !neonMatch[1].includes('postgresql://user:password') &&
      neonMatch[1].trim() !== '') {
    log(`  ℹ️  DATABASE_URL_NEON already configured`, colors.cyan);
    log(`     ${neonMatch[1].substring(0, 60)}...`, colors.cyan);
  } else {
    log('Enter your NeonDB connection string (or press Enter to skip):', colors.yellow);
    log('Example: postgresql://user:password@ep-xxx.region.neon.tech/db?sslmode=require', colors.reset);
    const neonUrl = await promptUser('> ');
    
    if (neonUrl && neonUrl.trim().length > 0) {
      let finalUrl = neonUrl.trim();
      
      // Add sslmode=require if not present
      if (!finalUrl.includes('sslmode=require')) {
        finalUrl += finalUrl.includes('?') ? '&sslmode=require' : '?sslmode=require';
      }
      
      content = updateOrAddEnvVar(content, 'DATABASE_URL_NEON', finalUrl);
      log(`  ✅ Set DATABASE_URL_NEON`, colors.green);
    } else {
      log('  ℹ️  Skipped NeonDB configuration', colors.cyan);
    }
  }
  
  // 3. Write back to file
  fs.writeFileSync(envPath, content, 'utf8');
  log(`\n✅ Updated .env file`, colors.green);
  
  return true;
}

async function main() {
  try {
    await fixEnvConfiguration();
    
    log(`\n${colors.bright}${colors.green}✅ Configuration fixed!${colors.reset}\n`);
    log('Run this to verify:', colors.cyan);
    log('  node scripts/setup/verify-config.js', colors.bright);
    log('');
    
  } catch (error) {
    log(`\n❌ Failed: ${error.message}`, colors.red);
    console.error(error);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}

module.exports = { fixEnvConfiguration };

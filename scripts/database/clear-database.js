#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const path = require('path');
const readline = require('readline');

// ANSI color codes
const colors = {
  cyan: '\x1b[0;36m',
  green: '\x1b[0;32m',
  red: '\x1b[0;31m',
  yellow: '\x1b[1;33m',
  reset: '\x1b[0m'
};

// Default values
const args = {
  databaseUrl: 'postgresql://localhost:5432/conhub',
  databaseName: 'conhub',
  username: 'postgres',
  password: '',
  databaseHost: 'localhost',
  port: '5432',
  confirm: false
};

// Parse command-line arguments
for (let i = 2; i < process.argv.length; i++) {
  const flag = process.argv[i];

  if (flag === '-Confirm') {
    args.confirm = true;
  } else {
    const value = process.argv[i + 1];
    switch (flag) {
      case '-DatabaseUrl':
        args.databaseUrl = value;
        i++;
        break;
      case '-DatabaseName':
        args.databaseName = value;
        i++;
        break;
      case '-Username':
        args.username = value;
        i++;
        break;
      case '-Password':
        args.password = value;
        i++;
        break;
      case '-DatabaseHost':
        args.databaseHost = value;
        i++;
        break;
      case '-Port':
        args.port = value;
        i++;
        break;
      default:
        if (!flag.startsWith('-')) continue;
        console.error(`Unknown parameter: ${flag}`);
        process.exit(1);
    }
  }
}

console.log(`${colors.yellow}[DATABASE] Clearing ConHub database...${colors.reset}`);

async function promptConfirm() {
  return new Promise((resolve) => {
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout
    });

    rl.question("Are you sure you want to delete ALL data? This cannot be undone! (type 'yes' to confirm) ", (answer) => {
      rl.close();
      resolve(answer === 'yes');
    });
  });
}

async function main() {
  if (!args.confirm) {
    const confirmed = await promptConfirm();
    if (!confirmed) {
      console.log(`${colors.red}[CANCELLED] Database clearing cancelled by user${colors.reset}`);
      process.exit(0);
    }
  }

  // Check if psql is available
  try {
    execSync('psql --version', { stdio: 'ignore' });
  } catch (error) {
    console.log(`${colors.red}[ERROR] PostgreSQL client (psql) not found. Please install PostgreSQL or add it to PATH.${colors.reset}`);
    process.exit(1);
  }

  console.log(`${colors.cyan}[EXECUTING] Running database cleanup script...${colors.reset}`);
  console.log(`${colors.yellow}[CONNECTION] Connecting to: ${args.databaseHost}:${args.port} as user: ${args.username}${colors.reset}`);

  const scriptDir = __dirname;
  const sqlFile = path.join(scriptDir, 'clear-database.sql');

  const env = { ...process.env, PGPASSWORD: args.password };
  const psql = spawn('psql', [
    '-h', args.databaseHost,
    '-p', args.port,
    '-U', args.username,
    '-d', args.databaseName,
    '-f', sqlFile
  ], { env, stdio: 'inherit' });

  psql.on('close', (code) => {
    if (code === 0) {
      console.log(`${colors.green}[SUCCESS] Database cleared successfully!${colors.reset}`);
      console.log(`${colors.green}[COMPLETE] Database cleanup completed${colors.reset}`);
    } else {
      console.log(`${colors.red}[ERROR] Failed to clear database${colors.reset}`);
    }
    process.exit(code);
  });
}

main();

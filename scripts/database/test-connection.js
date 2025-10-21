#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

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
  username: 'postgres',
  password: '',
  dbHost: 'localhost',
  port: '5432',
  database: 'postgres'
};

// Parse command-line arguments
for (let i = 2; i < process.argv.length; i += 2) {
  const flag = process.argv[i];
  const value = process.argv[i + 1];

  switch (flag) {
    case '-Username':
      args.username = value;
      break;
    case '-Password':
      args.password = value;
      break;
    case '-DbHost':
      args.dbHost = value;
      break;
    case '-Port':
      args.port = value;
      break;
    case '-Database':
      args.database = value;
      break;
    default:
      console.error(`Unknown parameter: ${flag}`);
      process.exit(1);
  }
}

console.log(`${colors.cyan}[TEST] Testing PostgreSQL Connection${colors.reset}`);
console.log(`Host: ${args.dbHost}`);
console.log(`Port: ${args.port}`);
console.log(`Username: ${args.username}`);
console.log(`Database: ${args.database}`);
console.log('');

console.log(`${colors.yellow}[TESTING] Attempting connection...${colors.reset}`);

const env = { ...process.env, PGPASSWORD: args.password };
const psql = spawn('psql', [
  '-h', args.dbHost,
  '-p', args.port,
  '-U', args.username,
  '-d', args.database,
  '-c', 'SELECT version();'
], { env });

let output = '';
let error = '';

psql.stdout.on('data', (data) => {
  output += data.toString();
});

psql.stderr.on('data', (data) => {
  error += data.toString();
});

psql.on('close', (code) => {
  if (code === 0) {
    console.log(`${colors.green}[SUCCESS] Connection successful!${colors.reset}`);
    console.log(output);
    console.log('');
    console.log(`${colors.cyan}You can now use these parameters:${colors.reset}`);
    console.log(`./clear-database.sh -Username '${args.username}' -Password '${args.password}'`);
  } else {
    console.log(`${colors.red}[FAILED] Connection failed${colors.reset}`);
    console.log(`${colors.red}${error}${colors.reset}`);
    console.log('');
    console.log(`${colors.yellow}Try these solutions:${colors.reset}`);
    console.log('1. Check if PostgreSQL service is running');
    console.log('2. Try different username (postgres, admin, your_username)');
    console.log("3. Try empty password: ./clear-database.sh -Password ''");
    console.log('4. Use Docker: ./start-docker-postgres.sh');
  }

  process.exit(code);
});

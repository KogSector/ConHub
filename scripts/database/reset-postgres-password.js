#!/usr/bin/env node

const { spawn, execSync } = require('child_process');
const readline = require('readline');

// ANSI color codes
const colors = {
  cyan: '\x1b[0;36m',
  green: '\x1b[0;32m',
  red: '\x1b[0;31m',
  yellow: '\x1b[1;33m',
  reset: '\x1b[0m'
};

const USERNAME = 'postgres';

console.log(`${colors.cyan}[POSTGRES] PostgreSQL Password Reset Helper${colors.reset}`);
console.log('');

// Check if psql is available
try {
  execSync('psql --version', { stdio: 'ignore' });
  const pgPath = execSync('which psql || where psql', { encoding: 'utf8' }).trim();
  console.log(`${colors.green}[INFO] PostgreSQL found at: ${pgPath}${colors.reset}`);
} catch (error) {
  console.log(`${colors.red}[ERROR] PostgreSQL not found in PATH. Please ensure PostgreSQL is installed.${colors.reset}`);
  console.log(`${colors.yellow}Common installation paths:${colors.reset}`);
  console.log('  - /usr/lib/postgresql/15/bin');
  console.log('  - /usr/lib/postgresql/14/bin');
  console.log('  - /usr/lib/postgresql/13/bin');
  process.exit(1);
}

console.log('');
console.log(`${colors.cyan}Choose an option:${colors.reset}`);
console.log('1. Try common default passwords');
console.log('2. Reset password using pg_hba.conf (requires service restart)');
console.log('3. Connect without password (trust method)');
console.log('4. Use different connection method');
console.log('');

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

rl.question('Enter your choice (1-4): ', async (choice) => {
  rl.close();

  switch (choice) {
    case '1':
      await tryCommonPasswords();
      break;
    case '2':
      showResetInstructions();
      break;
    case '3':
      await tryNoPassword();
      break;
    case '4':
      showAlternativeMethods();
      break;
    default:
      console.log(`${colors.red}[ERROR] Invalid choice${colors.reset}`);
  }

  console.log('');
  console.log(`${colors.cyan}[TIP] You can also try:${colors.reset}`);
  console.log("./clear-database.sh -Username 'your_username' -Password 'your_password'");
});

async function tryCommonPasswords() {
  console.log(`${colors.cyan}[TESTING] Trying common default passwords...${colors.reset}`);

  const commonPasswords = ['', 'postgres', 'admin', 'password', 'root', '123456'];

  for (const password of commonPasswords) {
    console.log(`${colors.yellow}Trying password: '${password}'${colors.reset}`);

    const env = { ...process.env, PGPASSWORD: password };
    try {
      execSync(`psql -h localhost -U ${USERNAME} -d postgres -c "SELECT version();"`, {
        env,
        stdio: 'ignore'
      });

      console.log(`${colors.green}[SUCCESS] Password found: '${password}'${colors.reset}`);
      console.log(`${colors.green}You can now use: ./clear-database.sh -Password '${password}'${colors.reset}`);
      return;
    } catch (error) {
      // Password didn't work, try next one
    }
  }

  console.log(`${colors.red}[FAILED] None of the common passwords worked${colors.reset}`);
}

function showResetInstructions() {
  console.log(`${colors.cyan}[INFO] To reset password using pg_hba.conf:${colors.reset}`);
  console.log("1. Stop PostgreSQL service (e.g., sudo systemctl stop postgresql)");
  console.log("2. Edit pg_hba.conf and change 'md5' or 'scram-sha-256' to 'trust' for local connections");
  console.log("3. Start PostgreSQL service (e.g., sudo systemctl start postgresql)");
  console.log("4. Connect and change password: ALTER USER postgres PASSWORD 'newpassword';");
  console.log("5. Change pg_hba.conf back to its original setting");
  console.log("6. Restart service");
  console.log('');
  console.log(`${colors.yellow}pg_hba.conf location: /etc/postgresql/[version]/main/pg_hba.conf${colors.reset}`);
}

async function tryNoPassword() {
  console.log(`${colors.cyan}[TESTING] Trying to connect without password...${colors.reset}`);

  try {
    execSync(`psql -h localhost -U ${USERNAME} -d postgres -c "SELECT version();"`, {
      stdio: 'ignore'
    });

    console.log(`${colors.green}[SUCCESS] Connected without password!${colors.reset}`);
    console.log(`${colors.green}You can now run: ./clear-database.sh -Password ''${colors.reset}`);
  } catch (error) {
    console.log(`${colors.red}[FAILED] Cannot connect without password${colors.reset}`);
  }
}

function showAlternativeMethods() {
  console.log(`${colors.cyan}[INFO] Alternative connection methods:${colors.reset}`);
  console.log('1. Use pgAdmin (GUI tool)');
  console.log('2. Use different username (try: postgres, admin, root)');
  console.log('3. Check if PostgreSQL is running on different port (5433, 5434, etc.)');
  console.log('4. Use Docker PostgreSQL: docker run -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres');
}

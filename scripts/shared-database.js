#!/usr/bin/env node

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

// Shared database configuration
const SHARED_DB = {
  host: 'localhost',
  port: 5432,
  database: 'conhub_shared',
  username: 'conhub_team',
  password: 'team_password_2024'
};

const DB_URL = `postgresql://${SHARED_DB.username}:${SHARED_DB.password}@${SHARED_DB.host}:${SHARED_DB.port}/${SHARED_DB.database}`;
const DUMP_FILE = path.join(__dirname, 'conhub_shared.sql');

function runCommand(command, options = {}) {
  try {
    return execSync(command, { stdio: 'inherit', ...options });
  } catch (error) {
    console.error(`❌ Command failed: ${command}`);
    process.exit(1);
  }
}

function setupSharedDatabase() {
  console.log('🚀 Setting up shared ConHub database...');
  
  // Create user and database
  const setupSQL = `
    -- Create user if not exists
    DO $$
    BEGIN
      IF NOT EXISTS (SELECT FROM pg_user WHERE usename = '${SHARED_DB.username}') THEN
        CREATE USER ${SHARED_DB.username} WITH PASSWORD '${SHARED_DB.password}';
      END IF;
    END
    $$;
    
    -- Create database if not exists
    SELECT 'CREATE DATABASE ${SHARED_DB.database} OWNER ${SHARED_DB.username}'
    WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '${SHARED_DB.database}')\\gexec
    
    -- Grant permissions
    GRANT ALL PRIVILEGES ON DATABASE ${SHARED_DB.database} TO ${SHARED_DB.username};
  `;
  
  // Execute setup as postgres user
  runCommand(`psql -U postgres -c "${setupSQL.replace(/\n/g, ' ')}"`);
  
  // Run migrations
  console.log('📋 Running database migrations...');
  process.env.DATABASE_URL = DB_URL;
  runCommand('cd database && sqlx migrate run');
  
  // Seed with development data
  seedDatabase();
  
  console.log('✅ Shared database setup complete!');
  console.log(`📍 Database URL: ${DB_URL}`);
  console.log('💡 Update your .env file with this DATABASE_URL');
}

function seedDatabase() {
  console.log('🌱 Seeding database with development data...');
  
  const seedData = `
    -- Users
    INSERT INTO users (user_id, email, name, created_at) VALUES
    ('user-1', 'alice@conhub.dev', 'Alice Developer', NOW()),
    ('user-2', 'bob@conhub.dev', 'Bob Engineer', NOW()),
    ('user-3', 'charlie@conhub.dev', 'Charlie Designer', NOW())
    ON CONFLICT (user_id) DO NOTHING;
    
    -- Repositories
    INSERT INTO repositories (id, name, url, owner_id, created_at) VALUES
    ('repo-1', 'conhub-frontend', 'https://github.com/team/conhub-frontend', 'user-1', NOW()),
    ('repo-2', 'conhub-backend', 'https://github.com/team/conhub-backend', 'user-2', NOW()),
    ('repo-3', 'shared-components', 'https://github.com/team/shared-components', 'user-1', NOW())
    ON CONFLICT (id) DO NOTHING;
    
    -- Documents
    INSERT INTO documents (id, title, content, source_type, owner_id, created_at) VALUES
    ('doc-1', 'API Documentation', 'ConHub API endpoints and usage', 'notion', 'user-1', NOW()),
    ('doc-2', 'Architecture Guide', 'System architecture overview', 'confluence', 'user-2', NOW()),
    ('doc-3', 'Development Setup', 'Local development environment setup', 'google_drive', 'user-3', NOW())
    ON CONFLICT (id) DO NOTHING;
  `;
  
  runCommand(`psql "${DB_URL}" -c "${seedData.replace(/\n/g, ' ')}"`);
}

function createDump() {
  console.log('📦 Creating database dump...');
  runCommand(`pg_dump "${DB_URL}" > "${DUMP_FILE}"`);
  console.log(`✅ Database dump created: ${DUMP_FILE}`);
  console.log('💡 Share this file with your team for database sync');
}

function restoreDump() {
  if (!fs.existsSync(DUMP_FILE)) {
    console.error(`❌ Dump file not found: ${DUMP_FILE}`);
    process.exit(1);
  }
  
  console.log('📥 Restoring database from dump...');
  
  // Drop and recreate database
  runCommand(`psql -U postgres -c "DROP DATABASE IF EXISTS ${SHARED_DB.database}"`);
  runCommand(`psql -U postgres -c "CREATE DATABASE ${SHARED_DB.database} OWNER ${SHARED_DB.username}"`);
  
  // Restore from dump
  runCommand(`psql "${DB_URL}" < "${DUMP_FILE}"`);
  
  console.log('✅ Database restored from dump');
}

function showStatus() {
  console.log('📊 ConHub Shared Database Status\n');
  
  try {
    const result = execSync(`psql "${DB_URL}" -c "SELECT 
      (SELECT count(*) FROM users) as users,
      (SELECT count(*) FROM repositories) as repositories,
      (SELECT count(*) FROM documents) as documents;"`, 
      { encoding: 'utf8' }
    );
    console.log(result);
    
    console.log(`🔗 Database URL: ${DB_URL}`);
    console.log(`📁 Dump file: ${fs.existsSync(DUMP_FILE) ? '✅ Available' : '❌ Not found'}`);
    
  } catch (error) {
    console.error('❌ Cannot connect to shared database');
    console.log('💡 Run: npm run db:setup');
  }
}

function updateEnvFile() {
  const envPath = path.join(__dirname, '..', '.env');
  let envContent = '';
  
  if (fs.existsSync(envPath)) {
    envContent = fs.readFileSync(envPath, 'utf8');
  }
  
  // Update or add DATABASE_URL
  const dbUrlPattern = /^DATABASE_URL.*$/m;
  const newDbUrl = `DATABASE_URL=${DB_URL}`;
  
  if (dbUrlPattern.test(envContent)) {
    envContent = envContent.replace(dbUrlPattern, newDbUrl);
  } else {
    envContent += `\n${newDbUrl}\n`;
  }
  
  fs.writeFileSync(envPath, envContent);
  console.log('✅ Updated .env file with shared database URL');
}

// Main command handler
const command = process.argv[2];

switch (command) {
  case 'setup':
    setupSharedDatabase();
    updateEnvFile();
    break;
    
  case 'dump':
    createDump();
    break;
    
  case 'restore':
    restoreDump();
    break;
    
  case 'status':
    showStatus();
    break;
    
  case 'sync':
    createDump();
    console.log('\n📤 To sync with team:');
    console.log('1. Share the conhub_shared.sql file');
    console.log('2. Team members run: npm run db:restore');
    break;
    
  default:
    console.log(`
🗄️  ConHub Shared Database Manager

Usage: npm run db:<command>

Commands:
  setup    - Create shared database with sample data
  dump     - Create database dump for sharing
  restore  - Restore database from dump file
  status   - Show database status and connection info
  sync     - Create dump and show sync instructions

Examples:
  npm run db:setup     # Initial setup
  npm run db:dump      # Before sharing changes
  npm run db:restore   # After receiving team updates
  npm run db:status    # Check current state
`);
    break;
}
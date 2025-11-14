const fs = require('fs');
const path = require('path');
const { Client } = require('pg');
require('dotenv').config({ path: '../data/.env' });

async function runMigrations() {
  const client = new Client({
    connectionString: process.env.DATABASE_URL
  });

  try {
    await client.connect();
    console.log('Connected to database');

    const migrationsDir = path.join(__dirname, '../database/migrations');
    const files = fs.readdirSync(migrationsDir).sort();

    for (const file of files) {
      if (file.endsWith('.sql')) {
        console.log(`Running migration: ${file}`);
        const sql = fs.readFileSync(path.join(migrationsDir, file), 'utf8');
        await client.query(sql);
        console.log(`✅ ${file} completed`);
      }
    }

    console.log('All migrations completed!');
  } catch (error) {
    console.error('Migration failed:', error);
  } finally {
    await client.end();
  }
}

runMigrations();
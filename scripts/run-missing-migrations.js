const fs = require('fs');
const path = require('path');
const { Client } = require('pg');
require('dotenv').config({ path: './data/.env' });

async function runMissingMigrations() {
  const client = new Client({
    connectionString: process.env.DATABASE_URL
  });

  try {
    await client.connect();
    console.log('Connected to database');

    // Run only the migrations that create the missing tables
    const migrations = [
      '006_create_connected_accounts_table.sql',
      '008_document_chunks_and_sync_jobs.sql'
    ];

    for (const file of migrations) {
      console.log(`Running migration: ${file}`);
      const sql = fs.readFileSync(path.join(__dirname, '../database/migrations', file), 'utf8');
      try {
        await client.query(sql);
        console.log(`✅ ${file} completed`);
      } catch (error) {
        if (error.code === '42P07') { // relation already exists
          console.log(`⚠️  ${file} - tables already exist, skipping`);
        } else {
          throw error;
        }
      }
    }

    console.log('Missing migrations completed!');
  } catch (error) {
    console.error('Migration failed:', error);
  } finally {
    await client.end();
  }
}

runMissingMigrations();
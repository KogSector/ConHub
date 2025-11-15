const fs = require('fs');
const path = require('path');
const { Client } = require('pg');
require('dotenv').config({ path: './data/.env' });

async function runSimpleMigration() {
  const client = new Client({
    connectionString: process.env.DATABASE_URL
  });

  try {
    await client.connect();
    console.log('Connected to database');

    const sql = fs.readFileSync(path.join(__dirname, 'create-missing-tables.sql'), 'utf8');
    await client.query(sql);
    console.log('✅ Missing tables created successfully');

  } catch (error) {
    console.error('Migration failed:', error);
  } finally {
    await client.end();
  }
}

runSimpleMigration();
const { Client } = require('pg');
const fs = require('fs');
const path = require('path');
require('dotenv').config();

async function resetDatabase() {
    const noRecreate = process.argv.includes('--no-recreate');
    const databaseUrl = process.env.DATABASE_URL_NEON || process.env.DATABASE_URL || process.env.DATABASE_URL_LOCAL;
    
    if (!databaseUrl) {
        console.error('❌ No database URL found');
        process.exit(1);
    }

    const client = new Client({
        connectionString: databaseUrl,
        ssl: databaseUrl.includes('neon.tech') ? { rejectUnauthorized: false } : false
    });

    try {
        await client.connect();
        console.log('🔌 Connected to database');

        // Drop all tables in correct order (respecting foreign keys)
        const dropTables = [
            'security_audit_log',
            'rate_limits',
            'api_keys',
            'oauth_connections',
            'email_verification_tokens',
            'password_reset_tokens',
            'user_sessions',
            'users'
        ];

        for (const table of dropTables) {
            try {
                await client.query(`DROP TABLE IF EXISTS ${table} CASCADE`);
                console.log(`✅ Dropped table: ${table}`);
            } catch (e) {
                console.log(`⚠️  Table ${table} not found or already dropped`);
            }
        }

        // Drop enums
        const dropEnums = [
            'audit_event_type',
            'session_status', 
            'subscription_tier',
            'user_role'
        ];

        for (const enumType of dropEnums) {
            try {
                await client.query(`DROP TYPE IF EXISTS ${enumType} CASCADE`);
                console.log(`✅ Dropped enum: ${enumType}`);
            } catch (e) {
                console.log(`⚠️  Enum ${enumType} not found`);
            }
        }

        console.log('🎉 Database completely nuked!');

        if (!noRecreate) {
            // Read and execute migration
            const migrationPath = path.join(__dirname, '../../auth/migrations/001_create_auth_tables.sql');
            const migration = fs.readFileSync(migrationPath, 'utf8');

            console.log('🏗️  Running database migration...');
            await client.query(migration);
            console.log('✅ Database schema created');

            // Verify tables exist
            const tables = await client.query(`
                SELECT table_name 
                FROM information_schema.tables 
                WHERE table_schema = 'public'
                ORDER BY table_name
            `);

            console.log('📊 Created tables:');
            tables.rows.forEach(row => console.log(`  - ${row.table_name}`));

            console.log('🎉 Database recreated successfully!');
        }

    } catch (error) {
        console.error('❌ Error:', error.message);
    } finally {
        await client.end();
    }
}

resetDatabase().catch(console.error);

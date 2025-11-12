const { Client } = require('pg');
const fs = require('fs');
const path = require('path');
require('dotenv').config();

async function clearRateLimits() {
    // Read database URL from environment
    const databaseUrl = process.env.DATABASE_URL_NEON || process.env.DATABASE_URL;
    
    if (!databaseUrl) {
        console.error('❌ DATABASE_URL_NEON or DATABASE_URL not found in environment');
        console.log('Please set your NeonDB connection string:');
        console.log('export DATABASE_URL_NEON="postgresql://user:password@ep-xxx.region.neon.tech/db?sslmode=require"');
        process.exit(1);
    }

    const client = new Client({
        connectionString: databaseUrl,
        ssl: {
            rejectUnauthorized: false
        }
    });

    try {
        console.log('🔌 Connecting to NeonDB...');
        await client.connect();
        console.log('✅ Connected to database');

        // Read the SQL file
        const sqlPath = path.join(__dirname, 'clear-rate-limits.sql');
        const sql = fs.readFileSync(sqlPath, 'utf8');

        console.log('🧹 Clearing rate limits and resetting security flags...');
        
        // Execute the cleanup
        const result = await client.query(sql);
        
        console.log('✅ Rate limits cleared successfully');
        console.log('✅ User security flags reset');
        
        // Check if there are any remaining rate limits
        const rateLimitsCheck = await client.query('SELECT COUNT(*) as count FROM rate_limits');
        const rateLimitCount = rateLimitsCheck.rows[0].count;
        
        console.log(`📊 Remaining rate limits: ${rateLimitCount}`);
        
        // Check users with security issues
        const usersCheck = await client.query(`
            SELECT COUNT(*) as count 
            FROM users 
            WHERE failed_login_attempts > 0 OR is_locked = TRUE OR locked_until IS NOT NULL
        `);
        const problemUsers = usersCheck.rows[0].count;
        
        console.log(`👥 Users with security issues: ${problemUsers}`);
        
        if (rateLimitCount === '0' && problemUsers === '0') {
            console.log('🎉 Database is clean! You can now try registration again.');
        }

    } catch (error) {
        console.error('❌ Error clearing rate limits:', error.message);
        
        if (error.message.includes('relation "rate_limits" does not exist')) {
            console.log('💡 The rate_limits table does not exist. Running migrations might be needed.');
        }
        
        process.exit(1);
    } finally {
        await client.end();
        console.log('🔌 Database connection closed');
    }
}

// Run the script
clearRateLimits().catch(console.error);
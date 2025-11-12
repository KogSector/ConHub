const { Client } = require('pg');
require('dotenv').config();

async function checkRateLimits() {
    const databaseUrl = process.env.DATABASE_URL_NEON || process.env.DATABASE_URL;
    
    if (!databaseUrl) {
        console.error('❌ DATABASE_URL_NEON or DATABASE_URL not found in environment');
        process.exit(1);
    }

    const client = new Client({
        connectionString: databaseUrl,
        ssl: { rejectUnauthorized: false }
    });

    try {
        await client.connect();
        console.log('✅ Connected to NeonDB');

        // Check rate limits
        console.log('\n📊 Current Rate Limits:');
        const rateLimits = await client.query(`
            SELECT 
                identifier,
                action,
                attempts,
                window_start,
                blocked_until,
                created_at
            FROM rate_limits 
            ORDER BY created_at DESC
        `);

        if (rateLimits.rows.length === 0) {
            console.log('✅ No rate limits found');
        } else {
            console.table(rateLimits.rows);
        }

        // Check users with issues
        console.log('\n👥 Users with Security Issues:');
        const users = await client.query(`
            SELECT 
                email,
                failed_login_attempts,
                is_locked,
                locked_until,
                last_login_at
            FROM users 
            WHERE failed_login_attempts > 0 OR is_locked = TRUE OR locked_until IS NOT NULL
        `);

        if (users.rows.length === 0) {
            console.log('✅ No users with security issues');
        } else {
            console.table(users.rows);
        }

        // Check recent security events
        console.log('\n🔍 Recent Security Events (last 10):');
        const events = await client.query(`
            SELECT 
                event_type,
                ip_address,
                details,
                created_at
            FROM security_audit_log 
            ORDER BY created_at DESC 
            LIMIT 10
        `);

        if (events.rows.length === 0) {
            console.log('✅ No recent security events');
        } else {
            console.table(events.rows);
        }

    } catch (error) {
        console.error('❌ Error:', error.message);
    } finally {
        await client.end();
    }
}

checkRateLimits().catch(console.error);
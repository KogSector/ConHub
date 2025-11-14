#!/usr/bin/env node

const fetch = require('node-fetch');

async function testBranchFetch() {
    const testPayload = {
        repoUrl: "https://github.com/microsoft/vscode",
        credentials: null
    };

    console.log('🧪 Testing branch fetch with payload:', JSON.stringify(testPayload, null, 2));

    try {
        console.log('📡 Making request to http://localhost:3013/api/data/sources/branches');
        
        const response = await fetch('http://localhost:3013/api/data/sources/branches', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(testPayload)
        });

        console.log('📊 Response status:', response.status);
        console.log('📊 Response headers:', Object.fromEntries(response.headers.entries()));

        const responseText = await response.text();
        console.log('📊 Raw response:', responseText);

        try {
            const result = JSON.parse(responseText);
            console.log('📊 Parsed response:', JSON.stringify(result, null, 2));
        } catch (e) {
            console.log('❌ Failed to parse JSON response');
        }

    } catch (error) {
        console.log('💥 Network Error:', error.message);
    }
}

async function checkHealth() {
    try {
        console.log('🏥 Checking data service health...');
        const response = await fetch('http://localhost:3013/health');
        const health = await response.json();
        console.log('🏥 Health check result:', JSON.stringify(health, null, 2));
        return true;
    } catch (error) {
        console.log('❌ Data service not running:', error.message);
        return false;
    }
}

async function main() {
    console.log('🚀 ConHub Branch Fetch Debug');
    console.log('============================');
    
    const isHealthy = await checkHealth();
    if (isHealthy) {
        await testBranchFetch();
    } else {
        console.log('💡 Start the data service with: npm run dev:data');
    }
}

main().catch(console.error);
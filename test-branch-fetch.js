#!/usr/bin/env node

const fetch = require('node-fetch');

async function testBranchFetch() {
    const testCases = [
        {
            name: "Public GitHub Repository",
            repoUrl: "https://github.com/microsoft/vscode",
            credentials: null
        },
        {
            name: "GitHub Repository with Token",
            repoUrl: "https://github.com/mvp-2003/Secrets",
            credentials: {
                accessToken: "your_github_token_here" // Replace with actual token
            }
        }
    ];

    for (const testCase of testCases) {
        console.log(`\n🧪 Testing: ${testCase.name}`);
        console.log(`📍 URL: ${testCase.repoUrl}`);
        
        try {
            const response = await fetch('http://localhost:3013/api/data/sources/branches', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    repoUrl: testCase.repoUrl,
                    credentials: testCase.credentials
                })
            });

            const result = await response.json();
            
            if (result.success) {
                console.log(`✅ Success! Found ${result.data.branches.length} branches`);
                console.log(`🌿 Default branch: ${result.data.defaultBranch || 'Not detected'}`);
                console.log(`📁 Branches: ${result.data.branches.slice(0, 5).join(', ')}${result.data.branches.length > 5 ? '...' : ''}`);
                
                if (result.data.file_extensions && result.data.file_extensions.length > 0) {
                    console.log(`📄 File types: ${result.data.file_extensions.slice(0, 10).join(', ')}${result.data.file_extensions.length > 10 ? '...' : ''}`);
                }
            } else {
                console.log(`❌ Failed: ${result.error || result.message}`);
            }
        } catch (error) {
            console.log(`💥 Network Error: ${error.message}`);
        }
    }
}

// Check if data service is running
async function checkService() {
    try {
        const response = await fetch('http://localhost:3013/health');
        const health = await response.json();
        console.log(`🏥 Data Service Health: ${health.status}`);
        console.log(`💾 Database: ${health.database}`);
        return true;
    } catch (error) {
        console.log(`❌ Data service not running on port 3013`);
        console.log(`💡 Start it with: npm run dev:data`);
        return false;
    }
}

async function main() {
    console.log('🚀 ConHub Branch Fetching Test');
    console.log('================================');
    
    const serviceRunning = await checkService();
    if (serviceRunning) {
        await testBranchFetch();
    }
    
    console.log('\n✨ Test completed!');
}

main().catch(console.error);
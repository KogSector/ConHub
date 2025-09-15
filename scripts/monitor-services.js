#!/usr/bin/env node
// Enhanced startup monitor to display URLs when services are ready

const { spawn } = require('child_process');
const net = require('net');

const SERVICES = [
    { name: 'Frontend (Next.js)', port: 3000, url: 'http://localhost:3000' },
    { name: 'Backend (Rust)', port: 3001, url: 'http://localhost:3001' },
    { name: 'LangChain Service', port: 3002, url: 'http://localhost:3002' },
    { name: 'Haystack Service', port: 8001, url: 'http://localhost:8001' }
];

function checkPort(port) {
    return new Promise((resolve) => {
        const sock = new net.Socket();
        sock.setTimeout(1000);
        
        sock.on('connect', () => {
            sock.destroy();
            resolve(true);
        });
        
        sock.on('timeout', () => {
            sock.destroy();
            resolve(false);
        });
        
        sock.on('error', () => {
            resolve(false);
        });
        
        sock.connect(port, 'localhost');
    });
}

async function checkAllServices() {
    const results = [];
    for (const service of SERVICES) {
        const isRunning = await checkPort(service.port);
        results.push({ ...service, running: isRunning });
    }
    return results;
}

function displayServiceUrls(services) {
    const running = services.filter(s => s.running);
    const stopped = services.filter(s => !s.running);
    
    if (running.length > 0) {
        console.log('\n🎉 ConHub Services Running:');
        console.log('━'.repeat(60));
        running.forEach(service => {
            console.log(`   ✅ ${service.name.padEnd(25)} ${service.url}`);
        });
        console.log('━'.repeat(60));
    }
    
    if (stopped.length > 0) {
        console.log('\n❌ Services Not Running:');
        stopped.forEach(service => {
            console.log(`   🔴 ${service.name.padEnd(25)} ${service.url} (Not running)`);
        });
    }
    
    if (running.length === SERVICES.length) {
        console.log('\n🚀 All services are ready! Open http://localhost:3000 to start using ConHub');
        console.log('💡 Press Ctrl+C to stop all services\n');
    }
}

async function monitorServices() {
    let servicesDisplayed = false;
    let checkCount = 0;
    const maxChecks = 20; // Check for up to 40 seconds
    
    const interval = setInterval(async () => {
        checkCount++;
        const services = await checkAllServices();
        const runningCount = services.filter(s => s.running).length;
        
        // Display URLs when all services are running or after reasonable time
        if ((runningCount === SERVICES.length || checkCount >= maxChecks) && !servicesDisplayed) {
            displayServiceUrls(services);
            servicesDisplayed = true;
            clearInterval(interval);
        }
    }, 2000); // Check every 2 seconds
}

// Start monitoring after a short delay to let concurrently output show first
setTimeout(monitorServices, 8000);

module.exports = { checkAllServices, displayServiceUrls };
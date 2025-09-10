const path = require('path')

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  // Load .env from root directory (excluding NODE_ENV which Next.js manages automatically)
  env: {
    ...(function() {
      const envVars = require('dotenv').config({ path: path.resolve(__dirname, '../.env') }).parsed || {};
      // Remove NODE_ENV as Next.js doesn't allow it to be explicitly set
      delete envVars.NODE_ENV;
      return envVars;
    })()
  }
}

module.exports = nextConfig
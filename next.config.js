const path = require('path')

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  // Load .env from root directory
  env: {
    ...(function() {
      const envVars = require('dotenv').config({ path: path.resolve(__dirname, '.env') }).parsed || {};
      delete envVars.NODE_ENV;
      return envVars;
    })()
  }
}

module.exports = nextConfig
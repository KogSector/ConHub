const path = require('path')

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  experimental: {
    // appDir is now default in Next.js 14 and this option is deprecated
  },
  // Disable static optimization for now
  trailingSlash: false,
  generateBuildId: async () => {
    return 'build-' + Date.now()
  },
  // Load .env from root directory
  env: {
    ...require('dotenv').config({ path: path.resolve(__dirname, '../.env') }).parsed
  }
}

module.exports = nextConfig
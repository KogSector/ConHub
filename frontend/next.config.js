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
  }
}

module.exports = nextConfig
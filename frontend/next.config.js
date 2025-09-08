/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  experimental: {
    appDir: true,
  },
  // Disable static optimization for now
  trailingSlash: false,
  generateBuildId: async () => {
    return 'build-' + Date.now()
  }
}

module.exports = nextConfig
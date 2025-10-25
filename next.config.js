const path = require('path');

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  webpack: (config) => {
    config.resolve.alias['@'] = path.join(__dirname, 'src/frontend');
    return config;
  },
  experimental: {
    turbo: {
      resolveAlias: {
        '@': './src/frontend',
      },
    },
  },
};

module.exports = nextConfig;

/** @type {import('next').NextConfig} */
const withPWA = require("next-pwa")({
  dest: "public",
});
const nextConfig = withPWA({
  output: "standalone",
  reactStrictMode: false,
  experimental: {
    scrollRestoration: true,
  },
});

export default nextConfig;

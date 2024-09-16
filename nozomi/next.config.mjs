/** @type {import('next').NextConfig} */
import pwa from "next-pwa";
const withPWA = pwa({
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

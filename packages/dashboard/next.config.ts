import type { NextConfig } from "next";

const config: NextConfig = {
  // Compile the workspace SDK (shipped as TypeScript source) on the fly.
  transpilePackages: ["@rustag/sdk"],
  eslint: {
    // The dashboard ships no ESLint config; type-checking still runs.
    ignoreDuringBuilds: true,
  },
};

export default config;

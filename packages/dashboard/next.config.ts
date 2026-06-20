import type { NextConfig } from "next";

const config: NextConfig = {
  // Compile the workspace SDK (shipped as TypeScript source) on the fly.
  transpilePackages: ["@rustag/sdk"],
  eslint: {
    // The dashboard ships no ESLint config; type-checking still runs.
    ignoreDuringBuilds: true,
  },
  // Every route is static, so we can ship a fully static export for hosting
  // (set NEXT_OUTPUT_EXPORT=1 at build time). Normal dev/build/start is unchanged.
  ...(process.env.NEXT_OUTPUT_EXPORT === "1" ? { output: "export" as const } : {}),
};

export default config;

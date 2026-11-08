import type { NextConfig } from "next";
import path from "node:path";

const nextConfig: NextConfig = {
  poweredByHeader: false,
  turbopack: {
    root: path.resolve(import.meta.dirname, ".."),
    resolveAlias: {
      react: "./website/node_modules/react",
      "react-dom": "./website/node_modules/react-dom",
    },
  },
  outputFileTracingIncludes: {
    "/updates/v1/*/*/latest.json": ["./updates/v1/stable/*/latest.json"],
  },
};

export default nextConfig;

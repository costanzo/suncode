import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  poweredByHeader: false,
  outputFileTracingIncludes: {
    "/updates/v1/*/*/latest.json": ["./updates/v1/stable/*/latest.json"],
  },
};

export default nextConfig;
